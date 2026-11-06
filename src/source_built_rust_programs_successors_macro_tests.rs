// Unit tests compiled with the emitted child-free helper (`rustc --test`).
#[cfg(test)]
mod macro_protocol_tests {
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;

    use super::*;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: PathBuf,
        original: Vec<String>,
        checkpoint: PathBuf,
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    fn fixture() -> Fixture {
        let root = std::env::temp_dir().join(format!(
            "mantle-macro-protocol-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let tmp = root.join("initial-tmp");
        std::fs::create_dir(&tmp).unwrap();
        std::fs::write(tmp.join("binary"), [0, 255, 1]).unwrap();
        let config = root.join("config");
        std::fs::write(&config, b"{\"arguments\":[],\"environment\":{}}\n/inputs/mrustc").unwrap();
        let image = root.join("image");
        std::fs::write(&image, b"exact Produced image").unwrap();
        let checkpoint = root.join("checkpoint");
        let original = vec![
            "--macro-control".into(),
            "-".into(),
            "-".into(),
            checkpoint.to_str().unwrap().into(),
            config.to_str().unwrap().into(),
            tmp.to_str().unwrap().into(),
            "2".into(),
            "derive-original".into(),
            image.to_str().unwrap().into(),
        ];
        let mut args = vec![
            "--mantle-controller-start".into(),
            checkpoint.to_str().unwrap().into(),
            "--".into(),
        ];
        args.extend(original.clone());
        assert_eq!(macro_start(&args).unwrap(), 0);
        Fixture {
            root,
            original,
            checkpoint,
        }
    }
    fn observation(f: &Fixture, name: &str, code: Option<u8>) -> PathBuf {
        let root = f.root.join(name);
        std::fs::create_dir(&root).unwrap();
        if let Some(code) = code {
            let outputs = root.join("outputs");
            std::fs::create_dir(&outputs).unwrap();
            copy(&f.checkpoint.join("workspace"), &outputs.join("workspace"), &mut 0).unwrap();
            std::fs::copy(f.checkpoint.join("times"), outputs.join("times")).unwrap();
            std::fs::create_dir(outputs.join("descriptor-files")).unwrap();
            for name in ["descriptors", "stderr"] {
                std::fs::write(outputs.join(name), b"").unwrap();
            }
            std::fs::write(outputs.join("stdout"), [0, 255, 3, 0]).unwrap();
            std::fs::write(outputs.join("exit-status"), format!("{code}\n")).unwrap();
            // Synthetic authenticated-envelope fixtures exercise parsing, not hashing.
            let fields = [
                "descriptor-files",
                "descriptors",
                "exit-status",
                "stderr",
                "stdout",
                "times",
                "workspace",
            ]
            .iter()
            .map(|name| format!("{}:{}", json_string(name), json_string(&"0".repeat(64))))
            .collect::<Vec<_>>()
            .join(",");
            std::fs::write(
                root.join("outcome.json"),
                format!("{{\"kind\":\"executed\",\"output_digests\":{{{fields}}}}}"),
            )
            .unwrap();
        } else {
            std::fs::write(root.join("outcome.json"), b"{\"kind\":\"skipped\",\"guard_codes\":[1]}").unwrap();
        }
        root
    }
    fn resume(f: &Fixture, out: &str, compiler: &Path, plugin: &Path) -> Vec<String> {
        let mut args = vec![
            "--mantle-controller-resume".into(),
            f.checkpoint.to_str().unwrap().into(),
            f.root.join(out).to_str().unwrap().into(),
            "1".into(),
            "2".into(),
            "3".into(),
            "2".into(),
            "compiler".into(),
            "0".into(),
            compiler.to_str().unwrap().into(),
            "macro".into(),
            "0".into(),
            plugin.to_str().unwrap().into(),
            "--".into(),
        ];
        args.extend(f.original.clone());
        args
    }
    #[test]
    fn compiler_actual_zero_finishes_with_binary_tmp_and_observed_times() {
        let f = fixture();
        let compiler = observation(&f, "compiler", Some(0));
        let plugin = observation(&f, "macro", None);
        let workspace = compiler.join("outputs/workspace");
        std::fs::write(workspace.join("translation/output"), [0, 254, 9]).unwrap();
        std::fs::write(workspace.join("tmp/binary"), [255, 0, 2]).unwrap();
        let observed = macro_times_capture(&workspace).unwrap();
        macro_times_write(&compiler.join("outputs/times"), &observed).unwrap();
        assert_eq!(macro_resume(&resume(&f, "done", &compiler, &plugin)).unwrap(), 1);
        let done = f.root.join("done");
        assert_eq!(macro_checkpoint(&done).unwrap().0, "done\n");
        assert_eq!(macro_times_read(&done.join("times")).unwrap(), observed);
        let finish = vec![
            "--mantle-controller-finish".into(),
            done.to_str().unwrap().into(),
            f.root.join("workspace").to_str().unwrap().into(),
            f.root.join("times").to_str().unwrap().into(),
            f.root.join("generated").to_str().unwrap().into(),
            f.root.join("export-checkpoint").to_str().unwrap().into(),
        ];
        assert_eq!(macro_finish(&finish).unwrap(), 0);
        assert_eq!(std::fs::read(f.root.join("workspace/tmp/binary")).unwrap(), [255, 0, 2]);
        assert_eq!(std::fs::read(f.root.join("workspace/translation/output")).unwrap(), [0, 254, 9]);
        assert!(macro_names(&f.root.join("generated")).unwrap().is_empty());
    }
    #[test]
    fn failed_missing_double_and_wrong_commands_never_complete() {
        let f = fixture();
        let failed = observation(&f, "failed", Some(1));
        let success = observation(&f, "success", Some(0));
        let skipped = observation(&f, "skipped", None);
        assert!(macro_resume(&resume(&f, "failed-next", &failed, &skipped)).is_err());
        assert!(macro_resume(&resume(&f, "missing-next", &skipped, &skipped)).is_err());
        assert!(macro_resume(&resume(&f, "double-next", &success, &success)).is_err());
        assert!(macro_resume(&resume(&f, "wrong-next", &skipped, &success)).is_err());
        let finish = vec![
            "--mantle-controller-finish".into(),
            f.checkpoint.to_str().unwrap().into(),
            "unused".into(),
            "unused".into(),
            "unused".into(),
            "unused".into(),
        ];
        assert!(macro_finish(&finish).is_err());
    }
    #[test]
    fn observed_envelopes_reject_forged_skips_and_unlisted_outputs() {
        let f = fixture();
        let skipped = observation(&f, "skipped", None);
        assert!(!macro_observation(&skipped).unwrap());
        std::fs::create_dir(skipped.join("outputs")).unwrap();
        assert!(macro_observation(&skipped).is_err());
        let executed = observation(&f, "executed", Some(0));
        assert!(macro_observation(&executed).unwrap());
        std::fs::write(executed.join("outputs/extra"), b"").unwrap();
        assert!(macro_observation(&executed).is_err());
        std::fs::write(skipped.join("outcome.json"), b"{\"kind\":\"skipped\",\"guard_codes\":[01]}").unwrap();
        assert!(macro_observation(&skipped).is_err());
    }
    #[test]
    fn exact_images_reject_drift_and_escaping_basenames() {
        let f = fixture();
        let other = f.root.join("other");
        std::fs::write(&other, std::fs::read(&f.original[8]).unwrap()).unwrap();
        assert!(macro_same_image(Path::new(&f.original[8]), &other).unwrap());
        std::fs::write(&other, b"different bytes").unwrap();
        assert!(!macro_same_image(Path::new(&f.original[8]), &other).unwrap());
        assert!(macro_inventory(&["../derive-original".into(), f.original[8].clone()]).is_err());
    }
    #[test]
    fn prepare_refreshes_translation_preserves_tmp_times_and_skips_macro() {
        let f = fixture();
        std::fs::write(f.checkpoint.join("workspace/translation/stale"), b"old").unwrap();
        let before = macro_times_capture(&f.checkpoint.join("workspace")).unwrap();
        macro_times_write(&f.checkpoint.join("times"), &before).unwrap();
        let make = |slot: &str, prefix: &str| {
            let mut args = vec![
                "--mantle-controller-prepare".into(),
                f.checkpoint.to_str().unwrap().into(),
                slot.into(),
            ];
            for name in ["invocation", "workspace", "stdin", "times", "data", "descriptor-files"] {
                args.push(f.root.join(format!("{prefix}-{name}")).to_str().unwrap().into());
            }
            args.extend([
                "compiler-config".into(),
                f.original[4].clone(),
                "--command-inputs".into(),
                "--runtime-inputs".into(),
                "0".into(),
                "--".into(),
            ]);
            if slot == "compiler" {
                args.extend(["fixed-image".into(), "compiler".into(), f.original[8].clone()]);
            } else {
                args.extend([
                    "inventory".into(),
                    "1".into(),
                    f.original[7].clone(),
                    f.original[8].clone(),
                ]);
            }
            args
        };
        assert_eq!(macro_prepare(&make("compiler", "selected")).unwrap(), 0);
        assert!(!f.root.join("selected-workspace/translation/stale").exists());
        assert_eq!(macro_times_read(&f.root.join("selected-times")).unwrap()["tmp/binary"], before["tmp/binary"]);
        assert_eq!(macro_prepare(&make("macro", "skipped")).unwrap(), 1);
        for name in ["invocation", "stdin", "times", "data"] {
            assert_eq!(std::fs::metadata(f.root.join(format!("skipped-{name}"))).unwrap().len(), 0);
        }
        assert!(macro_names(&f.root.join("skipped-workspace")).unwrap().is_empty());
        assert!(macro_names(&f.root.join("selected-descriptor-files")).unwrap().is_empty());
    }
    #[test]
    fn malformed_history_and_workspace_time_frames_fail_closed() {
        let f = fixture();
        std::fs::create_dir(f.checkpoint.join("workspace/history/1")).unwrap();
        assert!(macro_checkpoint(&f.checkpoint).is_err());
        std::fs::remove_dir(f.checkpoint.join("workspace/history/1")).unwrap();
        let mut times = macro_times_read(&f.checkpoint.join("times")).unwrap();
        times.remove("tmp/binary");
        macro_times_write(&f.root.join("missing-times"), &times).unwrap();
        assert!(macro_times_checked(&f.checkpoint.join("workspace"), &f.root.join("missing-times")).is_err());
        times.insert("tmp/binary".into(), (0, 1_000_000_000));
        macro_times_write(&f.root.join("invalid-times"), &times).unwrap();
        assert!(macro_times_read(&f.root.join("invalid-times")).is_err());
    }
    #[test]
    fn workspace_times_use_sorted_child_first_executor_order() {
        let f = fixture();
        let workspace = f.root.join("ordered-workspace");
        std::fs::create_dir_all(workspace.join("a/z")).unwrap();
        std::fs::write(workspace.join("a/file"), b"nested").unwrap();
        std::fs::write(workspace.join("a-b"), b"sibling").unwrap();
        let times = macro_times_capture(&workspace).unwrap();
        let frame = f.root.join("ordered-times");
        macro_times_write(&frame, &times).unwrap();
        let bytes = std::fs::read(frame).unwrap();
        let mut rest = bytes.as_slice();
        assert_eq!(macro_read_record(&mut rest).unwrap(), b"MANTLE-WORKSPACE-TIMES-1");
        let mut names = Vec::new();
        while !rest.is_empty() {
            names.push(std::str::from_utf8(macro_read_record(&mut rest).unwrap()).unwrap().to_owned());
            macro_read_record(&mut rest).unwrap();
            macro_read_record(&mut rest).unwrap();
        }
        assert_eq!(names, ["a/file", "a/z", "a", "a-b", ""]);
    }
}
