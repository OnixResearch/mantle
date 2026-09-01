#!/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/cc3hv2x3a5ywff41b7dx4l24aisplmkb-busybox-1.37.0-full-source-gcc10-v1/bin/sh
set -eu
STAGE_ID='mrustc-to-rust-1.90.0'
RUSTC_VERSION='1.90.0'
RUSTC_HOST_TRIPLE='x86_64-unknown-linux-musl'
RUSTC_TARGET='x86_64-unknown-linux-musl'
RUSTC_PROVIDER_TARGET_TRIPLE='x86_64-unknown-linux-musl'
MRUSTC_TARGET_VER='1.90'
SOURCE_DIR='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider-scratch/sources'
BUILD_DIR='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider-scratch/build'
OUTPUT_DIR='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider'
ARCHIVE_DIR='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider-scratch/archives'
MRUSTC_SOURCE_ID='mrustc-0.12.0'
RUST_SOURCE_ID='rust-1.90.0'
MRUSTC_SOURCE='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider-scratch/sources/mrustc-0.12.0'
RUST_SOURCE='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider-scratch/sources/rust-1.90.0'
MRUSTC_ARCHIVE='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider-scratch/archives/mrustc-0.12.0.tar.gz'
RUST_ARCHIVE='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider-scratch/archives/rust-1.90.0.tar.gz'
SOURCE_MANIFEST='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v69-native-prefix-resume-20260826.source-built-fixed-point-staging-2777399/rust-provider-scratch/mrustc-first-stage-sources.json'
MAKE_PROGRAM='make'
PARLEVEL=4
export PARLEVEL
COPY_PROGRAM='cp'
PKG_CONFIG_PROGRAM='pkg-config'
CMAKE_PROGRAM='cmake'
CC_PROGRAM='cc'
CXX_PROGRAM='c++'
HOST_GNU_TRIPLE='x86_64-unknown-linux-gnu'
MRUSTC_CXXFLAGS='-g0 -O2 -D_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_NONE'
TARGET_CC_PROGRAM='x86_64-unknown-linux-musl-gcc'
TARGET_CXX_PROGRAM='x86_64-unknown-linux-musl-g++'
TARGET_AR_PROGRAM='x86_64-unknown-linux-musl-ar'
TARGET_RANLIB_PROGRAM='x86_64-unknown-linux-musl-ranlib'
TARGET_MUSL_TOOL_PREFIXES='x86_64-unknown-linux-musl x86_64-linux-musl'
TARGET_MUSL_MACHINE_ALIASES='x86_64-unknown-linux-musl x86_64-linux-musl'
TARGET_MUSL_SOURCE_ROOT_SYSROOT='x86_64-linux-musl'
MANTLE_FULL_SOURCE_BOUND=true
MAKE_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/pka7fl1gzm5hmw295h0462cz46jg7nvc-make-4.4.1-full-source-gcc10-v1/bin/make'
CMAKE_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/b6nwg3slq2ka850yz7h9r8qd7s8dn31q-cmake-3.31.8-full-source-gcc10-v1/bin/cmake'
MANTLE_PYTHON_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/6gjbx73nihhd15j2inh3xp5fyxa58b3h-python-3.13.5-full-source-gcc10-v1/bin/python3.13'
PERL_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/a7fsq451mhx7kpa86ify8jac13g94xj4-perl-5.10.1-full-source-gcc10-v5/bin/perl'
PYTHONHOME='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/6gjbx73nihhd15j2inh3xp5fyxa58b3h-python-3.13.5-full-source-gcc10-v1'
PERL5LIB='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/a7fsq451mhx7kpa86ify8jac13g94xj4-perl-5.10.1-full-source-gcc10-v5/lib/5.10.1'
COPY_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/cc3hv2x3a5ywff41b7dx4l24aisplmkb-busybox-1.37.0-full-source-gcc10-v1/bin/cp'
SHELL_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/cc3hv2x3a5ywff41b7dx4l24aisplmkb-busybox-1.37.0-full-source-gcc10-v1/bin/sh'
CC_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/x86_64-linux-musl-gcc'
CXX_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/x86_64-linux-musl-g++'
TARGET_CC_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/x86_64-linux-musl-gcc'
TARGET_CXX_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/x86_64-linux-musl-g++'
TARGET_AR_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/x86_64-linux-musl-ar'
TARGET_RANLIB_PROGRAM='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/x86_64-linux-musl-ranlib'
MANTLE_TARGET_CC='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/x86_64-linux-musl-gcc'
MANTLE_TARGET_TOOLCHAIN_ROOT='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain'
SOURCE_ROOT='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain'
MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG='-B/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/libexec/gcc/x86_64-unknown-linux-musl/10.5.0/'
unset GCC_EXEC_PREFIX COMPILER_PATH
PATH='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/cc3hv2x3a5ywff41b7dx4l24aisplmkb-busybox-1.37.0-full-source-gcc10-v1/bin:/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/pka7fl1gzm5hmw295h0462cz46jg7nvc-make-4.4.1-full-source-gcc10-v1/bin:/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/b6nwg3slq2ka850yz7h9r8qd7s8dn31q-cmake-3.31.8-full-source-gcc10-v1/bin:/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/6gjbx73nihhd15j2inh3xp5fyxa58b3h-python-3.13.5-full-source-gcc10-v1/bin:/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/a7fsq451mhx7kpa86ify8jac13g94xj4-perl-5.10.1-full-source-gcc10-v5/bin:/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin'
SHELL="$SHELL_PROGRAM"
CONFIG_SHELL="$SHELL_PROGRAM"
GNUMAKEFLAGS="SHELL=$SHELL_PROGRAM"
PKG_CONFIG_PROGRAM=
MANTLE_ZLIB_HEADER='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/b6nwg3slq2ka850yz7h9r8qd7s8dn31q-cmake-3.31.8-full-source-gcc10-v1/include/zlib.h'
MANTLE_ZLIB_ARCHIVE='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/b6nwg3slq2ka850yz7h9r8qd7s8dn31q-cmake-3.31.8-full-source-gcc10-v1/lib/libz.a'
LIBRARY_PATH='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/b6nwg3slq2ka850yz7h9r8qd7s8dn31q-cmake-3.31.8-full-source-gcc10-v1/lib'
ZLIB_CFLAGS='-I/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/b6nwg3slq2ka850yz7h9r8qd7s8dn31q-cmake-3.31.8-full-source-gcc10-v1/include'
ZLIB_LIBS='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/b6nwg3slq2ka850yz7h9r8qd7s8dn31q-cmake-3.31.8-full-source-gcc10-v1/lib/libz.a'
MANTLE_LINUX_HEADERS_ROOT='/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/vhxjjhilrn43ivlcfw9sfzjmqvf1sz0g-linux-headers-6.6-full-source-gcc10-v3'
MANTLE_LINUX_HEADERS_CFLAGS='-I/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585/native-store/vhxjjhilrn43ivlcfw9sfzjmqvf1sz0g-linux-headers-6.6-full-source-gcc10-v3/include'
export PATH PYTHONHOME PERL5LIB LIBRARY_PATH MAKE_PROGRAM CMAKE_PROGRAM MANTLE_PYTHON_PROGRAM PERL_PROGRAM COPY_PROGRAM SHELL_PROGRAM SHELL CONFIG_SHELL GNUMAKEFLAGS CC_PROGRAM CXX_PROGRAM TARGET_CC_PROGRAM TARGET_CXX_PROGRAM TARGET_AR_PROGRAM TARGET_RANLIB_PROGRAM MANTLE_TARGET_CC MANTLE_TARGET_TOOLCHAIN_ROOT SOURCE_ROOT MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG MANTLE_ZLIB_HEADER MANTLE_ZLIB_ARCHIVE ZLIB_CFLAGS ZLIB_LIBS MANTLE_LINUX_HEADERS_ROOT MANTLE_LINUX_HEADERS_CFLAGS
printf '%s\n' 'using receipt-bound full-source Rust host tools with ambient discovery disabled'
printf '%s\n' "mantle rust source first stage: $STAGE_ID"
if [ ! -f "$SOURCE_MANIFEST" ]; then printf '%s\n' 'missing verified source manifest' >&2; exit 2; fi
if [ ! -d "$SOURCE_DIR"/'mrustc-0.12.0' ]; then printf '%s\n' 'missing verified source '\''mrustc-0.12.0'\'' from https://github.com/thepowersgang/mrustc/archive/refs/tags/v0.12.0.tar.gz with sha256 c1ba35f5fc5c4ca2952d9f5526e900dcb6632ea7fd4d71fa58029b3bb563ae56' >&2; exit 2; fi
if [ ! -d "$SOURCE_DIR"/'rust-1.90.0' ]; then printf '%s\n' 'missing verified source '\''rust-1.90.0'\'' from https://static.rust-lang.org/dist/rustc-1.90.0-src.tar.gz with sha256 799a9f9cba4ed5351e071048bcf6b5560755d9009648def33a407dd4961f9b7e' >&2; exit 2; fi
if [ ! -f "$MRUSTC_ARCHIVE" ]; then printf '%s\n' 'missing verified mrustc archive' >&2; exit 2; fi
if [ ! -f "$RUST_ARCHIVE" ]; then printf '%s\n' 'missing verified Rust archive' >&2; exit 2; fi
if [ ! -f "$MRUSTC_SOURCE/Makefile" ]; then printf '%s\n' 'mrustc source has no Makefile' >&2; exit 3; fi
if [ ! -f "$MRUSTC_SOURCE/minicargo.mk" ]; then printf '%s\n' 'mrustc source has no minicargo.mk' >&2; exit 3; fi
printf '%s\n' 'scrubbing inherited Cargo/build-script environment'
unset CARGO
unset CARGO_BIN_NAME
unset CARGO_BUILD_RUSTC_WRAPPER
unset CARGO_CFG_LINUX
unset CARGO_CFG_TARGET_ABI
unset CARGO_CFG_TARGET_ARCH
unset CARGO_CFG_TARGET_ENDIAN
unset CARGO_CFG_TARGET_ENV
unset CARGO_CFG_TARGET_FAMILY
unset CARGO_CFG_TARGET_HAS_ATOMIC
unset CARGO_CFG_TARGET_HAS_ATOMIC_EQUAL_ALIGNMENT
unset CARGO_CFG_TARGET_HAS_ATOMIC_LOAD_STORE
unset CARGO_CFG_TARGET_OS
unset CARGO_CFG_TARGET_POINTER_WIDTH
unset CARGO_CFG_TARGET_VENDOR
unset CARGO_CFG_UNIX
unset CARGO_CFG_WINDOWS
unset CARGO_CRATE_NAME
unset CARGO_ENCODED_RUSTFLAGS
unset CARGO_HOME
unset CARGO_MANIFEST_DIR
unset CARGO_MANIFEST_PATH
unset CARGO_PKG_AUTHORS
unset CARGO_PKG_DESCRIPTION
unset CARGO_PKG_HOMEPAGE
unset CARGO_PKG_LICENSE
unset CARGO_PKG_LICENSE_FILE
unset CARGO_PKG_NAME
unset CARGO_PKG_README
unset CARGO_PKG_REPOSITORY
unset CARGO_PKG_RUST_VERSION
unset CARGO_PKG_VERSION
unset CARGO_PKG_VERSION_MAJOR
unset CARGO_PKG_VERSION_MINOR
unset CARGO_PKG_VERSION_PATCH
unset CARGO_PKG_VERSION_PRE
unset CARGO_PRIMARY_PACKAGE
unset CARGO_TARGET_DIR
unset DEBUG
unset GCC_EXEC_PREFIX
unset COMPILER_PATH
unset HOST
unset MANTLE_RUST_CACHE_POLICY
unset MANTLE_RUSTC_MANIFEST
unset MANTLE_RUSTC_MANIFEST_DIR
unset MANTLE_RUSTC_MANIFEST_REF
unset MRUSTC_LIBDIR
unset NUM_JOBS
unset OPT_LEVEL
unset OUT_DIR
unset PROFILE
unset RUSTC
unset RUSTC_WORKSPACE_WRAPPER
unset RUSTC_WRAPPER
unset RUSTDOC
unset RUSTFLAGS
unset TARGET
if [ ! -x "$MAKE_PROGRAM" ]; then printf '%s\n' 'receipt-bound make is required for mrustc first stage' >&2; exit 3; fi
if [ ! -x "$COPY_PROGRAM" ]; then printf '%s\n' 'receipt-bound cp is required for mrustc first stage' >&2; exit 3; fi
if [ ! -x "$CMAKE_PROGRAM" ]; then printf '%s\n' 'receipt-bound cmake is required for mrustc first stage' >&2; exit 3; fi
if [ ! -x "$CC_PROGRAM" ]; then printf '%s\n' 'receipt-bound cc is required for mrustc first stage' >&2; exit 3; fi
if [ ! -x "$CXX_PROGRAM" ]; then printf '%s\n' 'receipt-bound c++ is required for mrustc first stage' >&2; exit 3; fi
if [ ! -x "$SHELL_PROGRAM" ]; then printf '%s\n' 'receipt-bound shell is required for mrustc first stage' >&2; exit 3; fi
if [ ! -x "$MANTLE_PYTHON_PROGRAM" ]; then printf '%s\n' 'receipt-bound Python is required for Rust bootstrap' >&2; exit 3; fi
if [ ! -x "$PERL_PROGRAM" ]; then printf '%s\n' 'receipt-bound Perl is required for Rust bootstrap' >&2; exit 3; fi
if [ ! -f "$MANTLE_ZLIB_HEADER" ]; then printf '%s\n' 'receipt-bound zlib header is required for mrustc first stage' >&2; exit 3; fi
if [ ! -f "$MANTLE_ZLIB_ARCHIVE" ]; then printf '%s\n' 'receipt-bound zlib archive is required for mrustc first stage' >&2; exit 3; fi
if [ ! -d "$PERL5LIB" ]; then printf '%s\n' 'receipt-bound Perl library is required for Rust bootstrap' >&2; exit 3; fi
if [ ! -d "$LIBRARY_PATH" ]; then printf '%s\n' 'receipt-bound zlib library directory is required for mrustc first stage' >&2; exit 3; fi
printf '%s\n' "verified sources manifest: $SOURCE_MANIFEST"
printf '%s\n' "build dir: $BUILD_DIR"
printf '%s\n' "output dir: $OUTPUT_DIR"
TEMP_ROOT="$BUILD_DIR/tmp"
mkdir -p "$BUILD_DIR" "$TEMP_ROOT"
export TMPDIR="$TEMP_ROOT"
export TMP="$TEMP_ROOT"
export TEMP="$TEMP_ROOT"
export TEMPDIR="$TEMP_ROOT"
$COPY_PROGRAM "$RUST_ARCHIVE" "$MRUSTC_SOURCE/rustc-${RUSTC_VERSION}-src.tar.gz"
cd "$MRUSTC_SOURCE"
mrustc_makefile=Makefile
mrustc_version_tmp="$BUILD_DIR/mrustc-authenticated-version.mk"
mrustc_version_matches=0
mrustc_version_replacement='	$V$(CXX) -o $@ -c $< $(CXXFLAGS) $(CPPFLAGS) -MMD -MP -MF $@.dep -D VERSION_GIT_FULLHASH=\"authenticated-mrustc-0.12.0\" -D VERSION_GIT_BRANCH=\"v0.12.0\" -D VERSION_GIT_SHORTHASH=\"v0.12.0\" -D VERSION_BUILDTIME=\"1970-01-01T00:00:00Z\" -D VERSION_GIT_ISDIRTY=0'
: > "$mrustc_version_tmp"
while IFS= read -r line; do
  if [ "$line" = "$mrustc_version_replacement" ]; then
    mrustc_version_matches=$((mrustc_version_matches + 1))
    printf '%s\n' "$line" >> "$mrustc_version_tmp"
  else
    case "$line" in
      *'-D VERSION_GIT_FULLHASH='*'-D VERSION_GIT_ISDIRTY='*) mrustc_version_matches=$((mrustc_version_matches + 1)); printf '%s\n' "$mrustc_version_replacement" >> "$mrustc_version_tmp" ;;
      *) printf '%s\n' "$line" >> "$mrustc_version_tmp" ;;
    esac
  fi
done < "$mrustc_makefile"
if [ "$mrustc_version_matches" -ne 1 ]; then printf '%s\n' 'mrustc Makefile must expose exactly one authenticated version metadata rule' >&2; exit 3; fi
$COPY_PROGRAM "$mrustc_version_tmp" "$mrustc_makefile"
rm -f "$mrustc_version_tmp"
mrustc_ivar_bounds_source=src/hir_typeck/expr_cs.cpp
mrustc_ivar_bounds_tmp="$BUILD_DIR/mrustc-ivar-bounds.tmp"
mrustc_ivar_bounds_matches=0
: > "${mrustc_ivar_bounds_tmp}"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = '                                context.possible_ivar_vals[te->index].force_disable = true;' ]; then
    printf '%s\n' '                                if(te->index >= context.possible_ivar_vals.size()) {' >> "${mrustc_ivar_bounds_tmp}"
    printf '%s\n' '                                    context.possible_ivar_vals.resize(te->index + 1);' >> "${mrustc_ivar_bounds_tmp}"
    printf '%s\n' '                                }' >> "${mrustc_ivar_bounds_tmp}"
    printf '%s\n' '                                context.possible_ivar_vals[te->index].force_disable = true;' >> "${mrustc_ivar_bounds_tmp}"
    mrustc_ivar_bounds_matches=$((${mrustc_ivar_bounds_matches} + 1))
  else
    printf '%s\n' "$line" >> "${mrustc_ivar_bounds_tmp}"
  fi
done < "${mrustc_ivar_bounds_source}"
if [ "${mrustc_ivar_bounds_matches}" -ne 1 ]; then printf '%s\n' 'authenticated Rust source rewrite mrustc-ivar-bounds did not match exactly once' >&2; exit 3; fi
$COPY_PROGRAM "${mrustc_ivar_bounds_tmp}" "${mrustc_ivar_bounds_source}"
rm -f "${mrustc_ivar_bounds_tmp}"
printf '%s\n' 'writing minicargo workspace boundary'
printf '%s\n' '[workspace]' > Cargo.toml
printf '%s\n' 'members = ["lib/libproc_macro"]' >> Cargo.toml
printf '%s\n' 'resolver = "2"' >> Cargo.toml
export RUSTC_TARGET MRUSTC_TARGET_VER RUSTC_VERSION
export RUSTC_INSTALL_BINDIR=bin
export OUTDIR_SUF=
export CARGO_CFG_RUSTIX_NO_LINUX_RAW=1
export CC="$CC_PROGRAM"
export CXX="$CXX_PROGRAM"
export CFLAGS="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_CFLAGS $MANTLE_LINUX_HEADERS_CFLAGS"
export CPPFLAGS="$ZLIB_CFLAGS $MANTLE_LINUX_HEADERS_CFLAGS"
export CXXFLAGS="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MRUSTC_CXXFLAGS $ZLIB_CFLAGS $MANTLE_LINUX_HEADERS_CFLAGS"
export LDFLAGS="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_LIBS"
export LIBS="$ZLIB_LIBS"
if [ "$RUSTC_HOST_TRIPLE" = "x86_64-unknown-linux-musl" ]; then
printf '%s\n' 'normalizing minicargo build-script OUT_DIR for static musl compiler host'
minicargo_build_source=tools/minicargo/build.cpp
if [ ! -f "$minicargo_build_source" ]; then printf '%s\n' 'minicargo build.cpp missing before musl host OUT_DIR normalization' >&2; exit 3; fi
minicargo_out_dir_tmp="$BUILD_DIR/minicargo-build.cpp"
minicargo_out_dir_original='    auto out_dir = parent.get_output_dir(m_is_for_host).to_absolute() / parent.get_build_script_out(m_manifest);'
minicargo_out_dir_marker='    if( m_manifest.build_script() != "" ) {'
minicargo_out_dir_patch_line='        out_dir = parent.get_output_dir(true).to_absolute() / parent.get_build_script_out(m_manifest);'
minicargo_out_dir_replaced=false
minicargo_out_dir_seen=false
minicargo_out_dir_pending=false
: > "$minicargo_out_dir_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$minicargo_out_dir_pending" = true ]; then
    if [ "$line" = "$minicargo_out_dir_marker" ]; then
      minicargo_out_dir_seen=true
    else
      printf '%s\n' "$minicargo_out_dir_marker" >> "$minicargo_out_dir_tmp"
      printf '%s\n' "$minicargo_out_dir_patch_line" >> "$minicargo_out_dir_tmp"
      printf '%s\n' '    }' >> "$minicargo_out_dir_tmp"
      minicargo_out_dir_replaced=true
    fi
    minicargo_out_dir_pending=false
  fi
  if [ "$line" = "$minicargo_out_dir_original" ]; then
    printf '%s\n' "$line" >> "$minicargo_out_dir_tmp"
    minicargo_out_dir_pending=true
  else
    printf '%s\n' "$line" >> "$minicargo_out_dir_tmp"
  fi
done < "$minicargo_build_source"
if [ "$minicargo_out_dir_pending" = true ]; then
  printf '%s\n' "$minicargo_out_dir_marker" >> "$minicargo_out_dir_tmp"
  printf '%s\n' "$minicargo_out_dir_patch_line" >> "$minicargo_out_dir_tmp"
  printf '%s\n' '    }' >> "$minicargo_out_dir_tmp"
  minicargo_out_dir_replaced=true
fi
if [ "$minicargo_out_dir_replaced" = false ] && [ "$minicargo_out_dir_seen" = false ]; then printf '%s\n' 'minicargo build.cpp lacks expected OUT_DIR line for musl host normalization' >&2; exit 3; fi
$COPY_PROGRAM "$minicargo_out_dir_tmp" "$minicargo_build_source"
rm -f "$minicargo_out_dir_tmp"
fi
if [ "$RUSTC_HOST_TRIPLE" = "x86_64-unknown-linux-musl" ]; then
printf '%s\n' 'normalizing minicargo rustc worker threads for static musl compiler host'
minicargo_threads_source=tools/minicargo/build.cpp
if [ ! -f "$minicargo_threads_source" ]; then printf '%s\n' 'minicargo build.cpp missing before rustc thread normalization' >&2; exit 3; fi
minicargo_threads_tmp="$BUILD_DIR/minicargo-rustc-threads-build.cpp"
minicargo_threads_anchor='        args.push_back("force-unstable-if-unmarked");'
minicargo_threads_marker_line='        // Mantle: keep static musl first-stage rustc single-threaded.'
minicargo_threads_flag_line='        args.push_back("-Z");'
minicargo_threads_value_line='        args.push_back("threads=1");'
minicargo_threads_inserted=false
minicargo_threads_pending=false
minicargo_threads_seen=false
: > "$minicargo_threads_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$minicargo_threads_pending" = true ]; then
    if [ "$line" = "$minicargo_threads_marker_line" ]; then
      minicargo_threads_seen=true
    else
      printf '%s\n' "$minicargo_threads_marker_line" >> "$minicargo_threads_tmp"
      printf '%s\n' "$minicargo_threads_flag_line" >> "$minicargo_threads_tmp"
      printf '%s\n' "$minicargo_threads_value_line" >> "$minicargo_threads_tmp"
      minicargo_threads_inserted=true
    fi
    minicargo_threads_pending=false
  fi
  printf '%s\n' "$line" >> "$minicargo_threads_tmp"
  if [ "$line" = "$minicargo_threads_anchor" ]; then minicargo_threads_pending=true; fi
done < "$minicargo_threads_source"
if [ "$minicargo_threads_pending" = true ]; then
  printf '%s\n' "$minicargo_threads_marker_line" >> "$minicargo_threads_tmp"
  printf '%s\n' "$minicargo_threads_flag_line" >> "$minicargo_threads_tmp"
  printf '%s\n' "$minicargo_threads_value_line" >> "$minicargo_threads_tmp"
  minicargo_threads_inserted=true
fi
if [ "$minicargo_threads_inserted" = false ] && [ "$minicargo_threads_seen" = false ]; then printf '%s\n' 'minicargo build.cpp lacks expected rustc force-unstable line for thread normalization' >&2; exit 3; fi
$COPY_PROGRAM "$minicargo_threads_tmp" "$minicargo_threads_source"
rm -f "$minicargo_threads_tmp"
fi
printf '%s\n' 'binding minicargo and Make recipes to protected absolute executables'
minicargo_makefile=minicargo.mk
minicargo_exec_tmp="$BUILD_DIR/minicargo-protected-exec.mk"
minicargo_exec_original='MINICARGO ?= bin/minicargo$(EXESUF)'
minicargo_exec_patched='MINICARGO ?= $(CURDIR)/bin/minicargo$(EXESUF)'
minicargo_exec_replaced=false
minicargo_exec_seen=false
: > "$minicargo_exec_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = "$minicargo_exec_original" ]; then
    printf '%s\n' "$minicargo_exec_patched" >> "$minicargo_exec_tmp"
    minicargo_exec_replaced=true
  else
    if [ "$line" = "$minicargo_exec_patched" ]; then minicargo_exec_seen=true; fi
    printf '%s\n' "$line" >> "$minicargo_exec_tmp"
  fi
done < "$minicargo_makefile"
if [ "$minicargo_exec_replaced" = false ] && [ "$minicargo_exec_seen" = false ]; then printf '%s\n' 'minicargo Makefile lacks the expected protected executable variable' >&2; exit 3; fi
$COPY_PROGRAM "$minicargo_exec_tmp" "$minicargo_makefile"
rm -f "$minicargo_exec_tmp"
run_rustc_makefile=run_rustc/Makefile
run_rustc_minicargo_tmp="$BUILD_DIR/run-rustc-minicargo.mk"
run_rustc_minicargo_original='MINICARGO ?= ../bin/minicargo$(EXESUF)'
run_rustc_minicargo_patched='MINICARGO ?= $(abspath ../bin/minicargo$(EXESUF))'
run_rustc_minicargo_replaced=false
run_rustc_minicargo_seen=false
: > "$run_rustc_minicargo_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = "$run_rustc_minicargo_original" ]; then
    printf '%s\n' "$run_rustc_minicargo_patched" >> "$run_rustc_minicargo_tmp"
    run_rustc_minicargo_replaced=true
  else
    if [ "$line" = "$run_rustc_minicargo_patched" ]; then run_rustc_minicargo_seen=true; fi
    printf '%s\n' "$line" >> "$run_rustc_minicargo_tmp"
  fi
done < "$run_rustc_makefile"
if [ "$run_rustc_minicargo_replaced" = false ] && [ "$run_rustc_minicargo_seen" = false ]; then printf '%s\n' 'run_rustc Makefile lacks the expected protected minicargo executable' >&2; exit 3; fi
$COPY_PROGRAM "$run_rustc_minicargo_tmp" "$run_rustc_makefile"
rm -f "$run_rustc_minicargo_tmp"
mrustc_codegen_source=src/trans/codegen_c.cpp
mrustc_codegen_tmp="$BUILD_DIR/codegen-c-bound-shell.cpp"
mrustc_system_matches=0
: > "$mrustc_codegen_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = '                int ec = system(cmd_ss.str().c_str());' ]; then
    printf '%s\n' '                // Mantle: execute compiler commands through the receipt-bound shell.' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                int ec = 0;' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                const char* mantle_shell = getenv("MRUSTC_SHELL");' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                if(mantle_shell != nullptr && mantle_shell[0] != '\''\0'\'')' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                {' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    pid_t child_pid = ::fork();' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    if(child_pid == -1)' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    {' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        ec = -1;' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    }' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    else if(child_pid == 0)' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    {' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        ::execl(mantle_shell, mantle_shell, "-c", cmd_ss.str().c_str(), static_cast<char*>(nullptr));' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        ::_exit(127);' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    }' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    else' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    {' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        int child_status = 0;' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        pid_t wait_result;' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        do' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        {' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                            wait_result = ::waitpid(child_pid, &child_status, 0);' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        } while(wait_result == -1 && errno == EINTR);' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                        ec = wait_result == -1 ? -1 : child_status;' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    }' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                }' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                else' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                {' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                    ec = system(cmd_ss.str().c_str());' >> "$mrustc_codegen_tmp"
    printf '%s\n' '                }' >> "$mrustc_codegen_tmp"
    mrustc_system_matches=$((mrustc_system_matches + 1))
  elif [ "$line" = '#include <cmath>' ]; then
    printf '%s\n' '#include <cmath>' >> "$mrustc_codegen_tmp"
    printf '%s\n' '#include <cerrno>' >> "$mrustc_codegen_tmp"
    printf '%s\n' '#include <cstdlib>' >> "$mrustc_codegen_tmp"
    printf '%s\n' '#include <sys/wait.h>' >> "$mrustc_codegen_tmp"
    printf '%s\n' '#include <unistd.h>' >> "$mrustc_codegen_tmp"
  else
    printf '%s\n' "$line" >> "$mrustc_codegen_tmp"
  fi
done < "$mrustc_codegen_source"
if [ "$mrustc_system_matches" -ne 1 ]; then printf '%s\n' 'mrustc codegen C source lacks one expected system call' >&2; exit 3; fi
$COPY_PROGRAM "$mrustc_codegen_tmp" "$mrustc_codegen_source"
rm -f "$mrustc_codegen_tmp"
mrustc_proc_macro_source=src/expand/proc_macro.cpp
mrustc_proc_macro_tmp="$BUILD_DIR/proc-macro-absolute.cpp"
mrustc_proc_macro_argv_matches=0
mrustc_proc_macro_spawn_matches=0
: > "$mrustc_proc_macro_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = '#include <unordered_set>' ]; then
    printf '%s\n' '#include <unordered_set>' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '#include <limits.h>' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '#include <stdlib.h>' >> "$mrustc_proc_macro_tmp"
  elif [ "$line" = '    char*   argv[3] = { const_cast<char*>(executable), const_cast<char*>(proc_macro_desc.name.c_str()), nullptr };' ]; then
    printf '%s\n' '    char resolved_executable[PATH_MAX];' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '    const char* spawn_executable = executable;' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '    if(executable[0] != '\''/'\'')' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '    {' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '        if(realpath(executable, resolved_executable) == nullptr)' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '        {' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '            BUG(sp, "Unable to resolve proc-macro executable `" << executable << "`, " << strerror(errno));' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '        }' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '        spawn_executable = resolved_executable;' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '    }' >> "$mrustc_proc_macro_tmp"
    printf '%s\n' '    char*   argv[3] = { const_cast<char*>(spawn_executable), const_cast<char*>(proc_macro_desc.name.c_str()), nullptr };' >> "$mrustc_proc_macro_tmp"
    mrustc_proc_macro_argv_matches=$((mrustc_proc_macro_argv_matches + 1))
  elif [ "$line" = '    int rv = posix_spawn(&this->handles.child_pid, executable, &file_actions, nullptr, argv, environ);' ]; then
    printf '%s\n' '    int rv = posix_spawn(&this->handles.child_pid, spawn_executable, &file_actions, nullptr, argv, environ);' >> "$mrustc_proc_macro_tmp"
    mrustc_proc_macro_spawn_matches=$((mrustc_proc_macro_spawn_matches + 1))
  else
    printf '%s\n' "$line" >> "$mrustc_proc_macro_tmp"
  fi
done < "$mrustc_proc_macro_source"
if [ "$mrustc_proc_macro_argv_matches" -ne 1 ] || [ "$mrustc_proc_macro_spawn_matches" -ne 1 ]; then printf '%s\n' 'mrustc proc-macro source lacks one expected spawn boundary' >&2; exit 3; fi
$COPY_PROGRAM "$mrustc_proc_macro_tmp" "$mrustc_proc_macro_source"
rm -f "$mrustc_proc_macro_tmp"
MANTLE_LLVM_RULE_NORMALIZER="$BUILD_DIR/normalize-llvm-rules"
printf '%s\n' "#!$SHELL_PROGRAM" > "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'set -eu' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'build_root=$1' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'busybox_dir=${SHELL_PROGRAM%/sh}' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'file_list=$build_root/.mantle-build-makefiles' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'files_seen=0' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'replacements=0' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'files_max=4096' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'replacements_max=4096' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '"$busybox_dir/find" "$build_root" -name build.make -type f > "$file_list"' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'while IFS= read -r file || [ -n "$file" ]; do' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '  files_seen=$((files_seen + 1))' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '  if [ "$files_seen" -gt "$files_max" ]; then echo '\''LLVM Makefile count exceeds bound'\'' >&2; exit 3; fi' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '  tmp=$file.mantle-absolute-exec' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '  : > "$tmp"' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '  while IFS= read -r line || [ -n "$line" ]; do' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '    for tool_name in llvm-min-tblgen llvm-tblgen; do' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '      for relative_prefix in ../../../../bin ../../../bin; do' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '        needle=$relative_prefix/$tool_name' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '        case "$line" in' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '        *"$needle"*)' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '          prefix=${line%%${needle}*}' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '          suffix=${line#*${needle}}' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '          line=${prefix}${build_root}/bin/${tool_name}${suffix}' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '          replacements=$((replacements + 1))' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '          if [ "$replacements" -gt "$replacements_max" ]; then echo '\''LLVM rule replacement count exceeds bound'\'' >&2; exit 3; fi' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '          ;;' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '        esac' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '      done' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '    done' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '    printf '\''%s\n'\'' "$line" >> "$tmp"' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '  done < "$file"' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '  "$busybox_dir/mv" "$tmp" "$file"' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'done < "$file_list"' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' '"$busybox_dir/rm" -f "$file_list"' >> "$MANTLE_LLVM_RULE_NORMALIZER"
printf '%s\n' 'if [ "$replacements" -eq 0 ]; then echo '\''LLVM generated rules contain no bounded tblgen paths'\'' >&2; exit 3; fi' >> "$MANTLE_LLVM_RULE_NORMALIZER"
chmod +x "$MANTLE_LLVM_RULE_NORMALIZER"
RECEIPT_MAKE_PROGRAM="$MAKE_PROGRAM"
receipt_make_dir="$BUILD_DIR/receipt-make-bin"
mkdir -p "$receipt_make_dir"
MAKE_PROGRAM="$receipt_make_dir/make"
printf '%s\n' "#!$SHELL_PROGRAM" > "$MAKE_PROGRAM"
printf '%s\n' "exec \"$RECEIPT_MAKE_PROGRAM\" \"SHELL=$SHELL_PROGRAM\" \"\$@\"" >> "$MAKE_PROGRAM"
chmod +x "$MAKE_PROGRAM"
printf '%s\n' "#!$SHELL_PROGRAM" > "$receipt_make_dir/sh"
printf '%s\n' "exec \"$SHELL_PROGRAM\" \"\$@\"" >> "$receipt_make_dir/sh"
chmod +x "$receipt_make_dir/sh"
PATH="$receipt_make_dir:$PATH"
MRUSTC_SHELL="$SHELL_PROGRAM"
export PATH MAKE_PROGRAM RECEIPT_MAKE_PROGRAM MRUSTC_SHELL MANTLE_LLVM_RULE_NORMALIZER
if [ "$RUSTC_HOST_TRIPLE" = "x86_64-unknown-linux-musl" ]; then
printf '%s\n' 'disabling LLVM shared-tool and execinfo backtraces for source-root musl host'
minicargo_makefile=minicargo.mk
if [ ! -f "$minicargo_makefile" ]; then printf '%s\n' 'minicargo Makefile missing before LLVM normalization' >&2; exit 3; fi
minicargo_llvm_tmp="$BUILD_DIR/minicargo-llvm-backtrace.mk"
minicargo_llvm_options_replaced=false
minicargo_llvm_options_seen_patched=false
minicargo_llvm_config_replaced=false
minicargo_llvm_config_seen_patched=false
minicargo_cmake_command_replaced=false
minicargo_cmake_command_seen_patched=false
minicargo_llvm_options_original='LLVM_CMAKE_OPTS += LLVM_ENABLE_ZLIB=OFF LLVM_ENABLE_TERMINFO=OFF LLVM_ENABLE_LIBEDIT=OFF WITH_POLLY=OFF'
minicargo_llvm_options_patched='LLVM_CMAKE_OPTS += LLVM_ENABLE_ZLIB=OFF LLVM_ENABLE_ZSTD=OFF LLVM_ENABLE_TERMINFO=OFF LLVM_ENABLE_LIBEDIT=OFF WITH_POLLY=OFF LLVM_ENABLE_BACKTRACES=OFF CMAKE_DISABLE_FIND_PACKAGE_Backtrace=ON CMAKE_DISABLE_FIND_PACKAGE_zstd=ON LLVM_TOOL_LTO_BUILD=OFF LLVM_BUILD_TOOLS=OFF'
minicargo_llvm_config_original='	$Vcd $(RUSTCSRC)build && $(MAKE) -j $(PARLEVEL)'
minicargo_llvm_config_patched='	$Vcd $(RUSTCSRC)build && $(MAKE) -j $(PARLEVEL) llvm-headers vt_gen llvm-config LLVMBinaryFormat LLVMMC LLVMAArch64Info LLVMBitstreamReader LLVMRemarks LLVMCore LLVMAArch64Utils LLVMCodeGenTypes LLVMAArch64Desc LLVMBitReader LLVMAsmParser LLVMIRReader LLVMMCParser LLVMTextAPI LLVMObject LLVMDebugInfoDWARF LLVMDebugInfoCodeView LLVMDebugInfoMSF LLVMDebugInfoPDB LLVMDebugInfoBTF LLVMSymbolize LLVMProfileData LLVMAnalysis LLVMBitWriter LLVMCGData LLVMTransformUtils LLVMObjCARCOpts LLVMAggressiveInstCombine LLVMInstCombine LLVMScalarOpts LLVMTarget LLVMCodeGen LLVMAsmPrinter LLVMCFGuard LLVMSelectionDAG LLVMGlobalISel LLVMSandboxIR LLVMVectorize LLVMAArch64CodeGen LLVMAArch64AsmParser LLVMMCDisassembler LLVMAArch64Disassembler LLVMARMInfo LLVMARMUtils LLVMARMDesc LLVMFrontendOffloading LLVMFrontendAtomic LLVMFrontendOpenMP LLVMLinker LLVMInstrumentation LLVMipo LLVMARMCodeGen LLVMARMAsmParser LLVMARMDisassembler LLVMCoverage LLVMExtensions LLVMCoroutines LLVMHipStdPar LLVMIRPrinter LLVMPasses LLVMLTO LLVMX86Info LLVMX86Desc LLVMX86CodeGen LLVMX86AsmParser LLVMX86Disassembler LLVMMCA LLVMX86TargetMCA'
minicargo_cmake_command_original='	$Vcd $(RUSTCSRC)build && cmake $(addprefix -D , $(LLVM_CMAKE_OPTS)) ../$(LLVM_DIR)'
minicargo_cmake_command_patched='	$Vcd $(RUSTCSRC)build && cmake $(addprefix -D , $(LLVM_CMAKE_OPTS)) ../$(LLVM_DIR) && $(MANTLE_LLVM_RULE_NORMALIZER) $(abspath $(RUSTCSRC)build)'
while IFS= read -r line; do
  if [ "$line" = "$minicargo_llvm_options_original" ]; then
    printf '%s\n' "$minicargo_llvm_options_patched" >> "$minicargo_llvm_tmp"
    minicargo_llvm_options_replaced=true
  elif [ "$line" = "$minicargo_llvm_config_original" ]; then
    printf '%s\n' "$minicargo_llvm_config_patched" >> "$minicargo_llvm_tmp"
    minicargo_llvm_config_replaced=true
  elif [ "$line" = "$minicargo_cmake_command_original" ]; then
    printf '%s\n' "$minicargo_cmake_command_patched" >> "$minicargo_llvm_tmp"
    minicargo_cmake_command_replaced=true
  else
    if [ "$line" = "$minicargo_llvm_options_patched" ]; then minicargo_llvm_options_seen_patched=true; fi
    if [ "$line" = "$minicargo_llvm_config_patched" ]; then minicargo_llvm_config_seen_patched=true; fi
    if [ "$line" = "$minicargo_cmake_command_patched" ]; then minicargo_cmake_command_seen_patched=true; fi
    printf '%s\n' "$line" >> "$minicargo_llvm_tmp"
  fi
done < "$minicargo_makefile"
if [ "$minicargo_llvm_options_replaced" = false ] && [ "$minicargo_llvm_options_seen_patched" = false ]; then printf '%s\n' 'minicargo Makefile lacks expected LLVM CMake options line for musl normalization' >&2; exit 3; fi
if [ "$minicargo_llvm_config_replaced" = false ] && [ "$minicargo_llvm_config_seen_patched" = false ]; then printf '%s\n' 'minicargo Makefile lacks expected llvm-config build line for musl normalization' >&2; exit 3; fi
if [ "$minicargo_cmake_command_replaced" = false ] && [ "$minicargo_cmake_command_seen_patched" = false ]; then printf '%s\n' 'minicargo Makefile lacks expected LLVM configure command for protected rule normalization' >&2; exit 3; fi
$COPY_PROGRAM "$minicargo_llvm_tmp" "$minicargo_makefile"
rm -f "$minicargo_llvm_tmp"
fi
$MAKE_PROGRAM -j "$PARLEVEL" CC="$CC" CXX="$CXX" CXXFLAGS="$CXXFLAGS" LDFLAGS="$LDFLAGS" LINKFLAGS_EXTRA="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" LIBS="$LIBS"
$MAKE_PROGRAM -j "$PARLEVEL" -f minicargo.mk CXXFLAGS_EXTRA="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" LINKFLAGS_EXTRA="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" bin/minicargo
if [ "$RUSTC_HOST_TRIPLE" = "x86_64-unknown-linux-musl" ] && [ "$RUSTC_PROVIDER_TARGET_TRIPLE" = "x86_64-unknown-linux-musl" ]; then
printf '%s\n' 'preparing source-root musl LLVM host compiler runtime'
RUSTC_TARGET="$RUSTC_HOST_TRIPLE"
if [ "$RUSTC_TARGET" = "$RUSTC_PROVIDER_TARGET_TRIPLE" ] && [ "$RUSTC_TARGET" = "x86_64-unknown-linux-musl" ]; then
if [ ! -x "$TARGET_CC_PROGRAM" ]; then printf '%s\n' 'receipt-bound x86_64 musl target gcc is missing' >&2; exit 3; fi
target_cc_path="$TARGET_CC_PROGRAM"
target_toolchain_root="$MANTLE_TARGET_TOOLCHAIN_ROOT"
target_cc_machine=$("$target_cc_path" -dumpmachine 2>/dev/null || true)
case " $TARGET_MUSL_MACHINE_ALIASES " in *" $target_cc_machine "*) ;; *) printf '%s\n' "target gcc machine expected one of: $TARGET_MUSL_MACHINE_ALIASES; got $target_cc_machine" >&2; exit 3 ;; esac
target_tool_dir=$(dirname "$target_cc_path")
target_wrapper_root=$(dirname "$target_tool_dir")
target_cc_basename=${target_cc_path##*/}
case "$target_cc_basename" in *-gcc) target_tool_prefix=${target_cc_basename%-gcc} ;; *) target_tool_prefix= ;; esac
if [ -z "$target_tool_prefix" ]; then printf '%s\n' "target gcc name does not end in -gcc: $target_cc_basename" >&2; exit 3; fi
target_cxx_program=$target_tool_prefix-g++
target_ar_program=$target_tool_prefix-ar
target_ranlib_program=$target_tool_prefix-ranlib
target_objcopy_program=$target_tool_prefix-objcopy
if [ -n "$target_toolchain_root" ]; then
  for target_required_tool in "$target_tool_dir/$target_cxx_program" "$target_tool_dir/$target_ar_program" "$target_tool_dir/$target_ranlib_program" "$target_tool_dir/$target_objcopy_program"; do
    if [ ! -x "$target_required_tool" ]; then printf '%s\n' "source-root musl target toolchain root is incomplete: $target_required_tool" >&2; exit 3; fi
  done
  for target_required_file in "$target_toolchain_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libc.so" "$target_toolchain_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libgcc_s.so.1"; do
    if [ ! -f "$target_required_file" ]; then printf '%s\n' "source-root musl target toolchain root is incomplete: $target_required_file" >&2; exit 3; fi
  done
fi
target_nix_support="$target_wrapper_root/nix-support"
target_orig_libc_file="$target_nix_support/orig-libc"
target_orig_cc_file="$target_nix_support/orig-cc"
if [ -f "$target_orig_libc_file" ] && [ -f "$target_orig_cc_file" ]; then
  target_libc_root=$(cat "$target_orig_libc_file")
  target_orig_cc_root=$(cat "$target_orig_cc_file")
  target_gcc_crt_machine=$RUSTC_TARGET
else
  target_libc_root="$target_wrapper_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT"
  target_orig_cc_root="$target_wrapper_root"
  target_gcc_crt_machine=$target_cc_machine
fi
target_musl_crt_dir="$target_libc_root/lib"
target_gcc_version=$("$target_cc_path" -dumpfullversion 2>/dev/null || "$target_cc_path" -dumpversion 2>/dev/null || true)
target_gcc_crt_dir="$target_orig_cc_root/lib/gcc/$target_gcc_crt_machine/$target_gcc_version"
if [ ! -f "$target_gcc_crt_dir/crtbeginS.o" ]; then for candidate_dir in "$target_orig_cc_root"/lib/gcc/"$target_gcc_crt_machine"/*; do if [ -f "$candidate_dir/crtbeginS.o" ]; then target_gcc_crt_dir="$candidate_dir"; break; fi; done; fi
if [ ! -f "$target_musl_crt_dir/crt1.o" ] || [ ! -f "$target_musl_crt_dir/Scrt1.o" ] || [ ! -f "$target_musl_crt_dir/rcrt1.o" ] || [ ! -f "$target_musl_crt_dir/crti.o" ] || [ ! -f "$target_musl_crt_dir/crtn.o" ] || [ ! -f "$target_gcc_crt_dir/crtbeginS.o" ] || [ ! -f "$target_gcc_crt_dir/libgcc.a" ]; then printf '%s\n' 'target gcc toolchain does not expose musl/gcc CRT and unwinder objects' >&2; exit 3; fi
export NIX_CC_WRAPPER_TARGET_HOST_x86_64_unknown_linux_musl=1
export COPY_PROGRAM
target_alias_dir="$BUILD_DIR/target-linker-bin"
target_runtime_dir="$BUILD_DIR/target-linker-runtime"
mkdir -p "$target_alias_dir" "$target_runtime_dir"
for crt_name in crt1.o Scrt1.o rcrt1.o crti.o crtn.o; do rm -f "$target_runtime_dir/$crt_name"; $COPY_PROGRAM "$target_musl_crt_dir/$crt_name" "$target_runtime_dir/$crt_name"; done
for crt_name in crtbeginS.o crtendS.o; do rm -f "$target_runtime_dir/$crt_name"; $COPY_PROGRAM "$target_gcc_crt_dir/$crt_name" "$target_runtime_dir/$crt_name"; done
if [ ! -x "$target_tool_dir/$target_objcopy_program" ]; then printf '%s\n' 'target gcc toolchain does not expose objcopy for CRT normalization' >&2; exit 3; fi
target_crtbegin_no_frame_init="$target_runtime_dir/crtbeginS.o.no-frame-init"
rm -f "$target_crtbegin_no_frame_init"
"$target_tool_dir/$target_objcopy_program" --remove-section .init_array --remove-section .rela.init_array --remove-section .fini_array --remove-section .rela.fini_array "$target_runtime_dir/crtbeginS.o" "$target_crtbegin_no_frame_init"
rm -f "$target_runtime_dir/crtbeginS.o"
$COPY_PROGRAM "$target_crtbegin_no_frame_init" "$target_runtime_dir/crtbeginS.o"
rm -f "$target_crtbegin_no_frame_init"
target_unwind_archive="$target_gcc_crt_dir/libgcc_eh.a"
if [ ! -f "$target_unwind_archive" ]; then target_unwind_archive="$target_gcc_crt_dir/libgcc.a"; fi
rm -f "$target_runtime_dir/libunwind.a"; $COPY_PROGRAM "$target_unwind_archive" "$target_runtime_dir/libunwind.a"
rm -f "$target_runtime_dir/libgcc.a"; $COPY_PROGRAM "$target_gcc_crt_dir/libgcc.a" "$target_runtime_dir/libgcc.a"
rm -f "$target_runtime_dir/libgcc_s.a" "$target_runtime_dir/libgcc_s.so" "$target_runtime_dir/libgcc_s.so.1"
$COPY_PROGRAM "$target_gcc_crt_dir/libgcc.a" "$target_runtime_dir/libgcc_s.a"
for target_libgcc_shared_name in libgcc_s.so libgcc_s.so.1; do
  for target_libgcc_shared_dir in "$target_musl_crt_dir" "$target_orig_cc_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib" "$target_gcc_crt_dir"; do
    if [ -f "$target_libgcc_shared_dir/$target_libgcc_shared_name" ]; then $COPY_PROGRAM "$target_libgcc_shared_dir/$target_libgcc_shared_name" "$target_runtime_dir/$target_libgcc_shared_name"; break; fi
  done
done
target_lfs_compat_source="$target_runtime_dir/musl-lfs-compat.c"
target_lfs_compat_object="$target_runtime_dir/musl-lfs-compat.o"
cat > "$target_lfs_compat_source" <<'MANTLE_MUSL_LFS_COMPAT_C'
#define _LARGEFILE64_SOURCE 1
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <sys/mman.h>
#include <sys/sendfile.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#ifdef fstat64
#undef fstat64
#endif
#ifdef fstatat64
#undef fstatat64
#endif
#ifdef ftruncate64
#undef ftruncate64
#endif
#ifdef sendfile64
#undef sendfile64
#endif
#ifdef lseek64
#undef lseek64
#endif
#ifdef lstat64
#undef lstat64
#endif
#ifdef mmap64
#undef mmap64
#endif
#ifdef open64
#undef open64
#endif
#ifdef openat64
#undef openat64
#endif
#ifdef pread64
#undef pread64
#endif
#ifdef pwrite64
#undef pwrite64
#endif
#ifdef readdir64
#undef readdir64
#endif
#ifdef stat64
#undef stat64
#endif

#define MANTLE_PTHREAD_TLS_KEY_CAPACITY 4096u

typedef void (*mantle_pthread_tls_destructor)(void *);

static int mantle_pthread_tls_used[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static mantle_pthread_tls_destructor mantle_pthread_tls_destructors[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static __thread void *mantle_pthread_tls_values[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static unsigned int mantle_pthread_tls_next_key;

static int mantle_pthread_tls_key_is_valid(pthread_key_t key) {
    unsigned int index = (unsigned int)key;
    if (index >= MANTLE_PTHREAD_TLS_KEY_CAPACITY) {
        return 0;
    }
    if (__sync_add_and_fetch(&mantle_pthread_tls_used[index], 0) == 0) {
        return 0;
    }
    return 1;
}

int __wrap_pthread_key_create(pthread_key_t *key, void (*destructor)(void *)) {
    if (key == NULL) {
        return EINVAL;
    }
    for (unsigned int offset = 0; offset < MANTLE_PTHREAD_TLS_KEY_CAPACITY; offset++) {
        unsigned int candidate = (mantle_pthread_tls_next_key + offset) % MANTLE_PTHREAD_TLS_KEY_CAPACITY;
        if (__sync_bool_compare_and_swap(&mantle_pthread_tls_used[candidate], 0, 1) != 0) {
            mantle_pthread_tls_destructors[candidate] = destructor;
            *key = (pthread_key_t)candidate;
            mantle_pthread_tls_next_key = (candidate + 1) % MANTLE_PTHREAD_TLS_KEY_CAPACITY;
            return 0;
        }
    }
    return EAGAIN;
}

int __wrap_pthread_key_delete(pthread_key_t key) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return EINVAL;
    }
    unsigned int index = (unsigned int)key;
    mantle_pthread_tls_values[index] = NULL;
    mantle_pthread_tls_destructors[index] = NULL;
    __sync_lock_release(&mantle_pthread_tls_used[index]);
    return 0;
}

void *__wrap_pthread_getspecific(pthread_key_t key) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return NULL;
    }
    return mantle_pthread_tls_values[(unsigned int)key];
}

int __wrap_pthread_setspecific(pthread_key_t key, const void *value) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return EINVAL;
    }
    mantle_pthread_tls_values[(unsigned int)key] = (void *)value;
    return 0;
}

int backtrace(void **buffer, int size) {
    (void)buffer;
    (void)size;
    return 0;
}

void backtrace_symbols_fd(void *const *buffer, int size, int fd) {
    (void)buffer;
    (void)size;
    (void)fd;
}

static int mantle_open_flags_need_mode(int flags) {
    if ((flags & O_CREAT) != 0) {
        return 1;
    }
#ifdef O_TMPFILE
    if ((flags & O_TMPFILE) == O_TMPFILE) {
        return 1;
    }
#endif
    return 0;
}

int fstat64(int fd, struct stat *buf) {
    return fstat(fd, buf);
}

int fstatat64(int dirfd, const char *pathname, struct stat *buf, int flags) {
    return fstatat(dirfd, pathname, buf, flags);
}

int ftruncate64(int fd, off_t length) {
    return ftruncate(fd, length);
}

ssize_t sendfile64(int out_fd, int in_fd, off_t *offset, size_t count) {
    return sendfile(out_fd, in_fd, offset, count);
}

off_t lseek64(int fd, off_t offset, int whence) {
    return lseek(fd, offset, whence);
}

int lstat64(const char *pathname, struct stat *buf) {
    return lstat(pathname, buf);
}

void *mmap64(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    return mmap(addr, length, prot, flags, fd, offset);
}

int open64(const char *pathname, int flags, ...) {
    mode_t mode = 0;
    if (mantle_open_flags_need_mode(flags) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
        return open(pathname, flags, mode);
    }
    return open(pathname, flags);
}

int openat64(int dirfd, const char *pathname, int flags, ...) {
    mode_t mode = 0;
    if (mantle_open_flags_need_mode(flags) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
        return openat(dirfd, pathname, flags, mode);
    }
    return openat(dirfd, pathname, flags);
}

ssize_t pread64(int fd, void *buf, size_t count, off_t offset) {
    return pread(fd, buf, count, offset);
}

ssize_t pwrite64(int fd, const void *buf, size_t count, off_t offset) {
    return pwrite(fd, buf, count, offset);
}

struct dirent *readdir64(DIR *dirp) {
    return readdir(dirp);
}

int stat64(const char *pathname, struct stat *buf) {
    return stat(pathname, buf);
}
MANTLE_MUSL_LFS_COMPAT_C
"$target_cc_path" "$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" -D_LARGEFILE64_SOURCE -fno-asynchronous-unwind-tables -fPIC -c "$target_lfs_compat_source" -o "$target_lfs_compat_object"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/sh"
printf '%s\n' "exec \"$SHELL_PROGRAM\" \"\$@\"" >> "$target_alias_dir/sh"
chmod +x "$target_alias_dir/sh"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/emcc"
printf '%s\n' 'exit 127' >> "$target_alias_dir/emcc"
chmod +x "$target_alias_dir/emcc"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/pkg-config"
chmod +x "$target_alias_dir/pkg-config"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/pkgconf"
chmod +x "$target_alias_dir/pkgconf"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/git"
chmod +x "$target_alias_dir/git"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/perl"
printf '%s\n' "exec \"$PERL_PROGRAM\" \"\$@\"" >> "$target_alias_dir/perl"
chmod +x "$target_alias_dir/perl"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/make"
printf '%s\n' "exec \"$MAKE_PROGRAM\" \"\$@\"" >> "$target_alias_dir/make"
chmod +x "$target_alias_dir/make"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/cc"
printf '%s\n' 'set -eu' >> "$target_alias_dir/cc"
printf '%s\n' "target_cc_path=\"\${MANTLE_TARGET_CC_PATH:-$target_cc_path}\"" >> "$target_alias_dir/cc"
printf '%s\n' "target_runtime_dir=\"$target_runtime_dir\"" >> "$target_alias_dir/cc"
printf '%s\n' 'static_pie_normalized=false' >> "$target_alias_dir/cc"
printf '%s\n' 'output_path=' >> "$target_alias_dir/cc"
printf '%s\n' 'previous_arg=' >> "$target_alias_dir/cc"
printf '%s\n' 'rustc_main_response_file=false' >> "$target_alias_dir/cc"
printf '%s\n' 'shared_link=false' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  if [ "$previous_arg" = "-o" ]; then output_path="$arg"; previous_arg=; continue; fi' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    -static-pie) static_pie_normalized=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -shared|-dynamiclib) shared_link=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -o) previous_arg=-o ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    @*/output/rustc-build/rustc_main_cmd.txt|@output/rustc-build/rustc_main_cmd.txt) rustc_main_response_file=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'dynamic_rustc_link=false' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$rustc_main_response_file" = true ]; then dynamic_rustc_link=true; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'case "$output_path" in' >> "$target_alias_dir/cc"
printf '%s\n' '  */output/rustc|output/rustc|*/output/rustc-build/rustc_main|output/rustc-build/rustc_main) dynamic_rustc_link=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' 'esac' >> "$target_alias_dir/cc"
printf '%s\n' 'mapped_args_set=false' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    rcrt1.o|*/rcrt1.o) mapped_arg="$target_runtime_dir/crt1.o" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    crt1.o|*/crt1.o|Scrt1.o|*/Scrt1.o|crti.o|*/crti.o|crtn.o|*/crtn.o|crtbeginS.o|*/crtbeginS.o|crtendS.o|*/crtendS.o) mapped_arg="$target_runtime_dir/${arg##*/}" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -static) if [ "$shared_link" = true ]; then continue; else mapped_arg="$arg"; fi ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -static-pie) if [ "$dynamic_rustc_link" = true ] || [ "$shared_link" = true ]; then continue; else mapped_arg="-static"; fi ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    *) mapped_arg="$arg" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' '  if [ "$mapped_args_set" = false ]; then set -- "$mapped_arg"; mapped_args_set=true; else set -- "$@" "$mapped_arg"; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$mapped_args_set" = false ]; then set --; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'link_command=true' >> "$target_alias_dir/cc"
printf '%s\n' 'static_support_link=true' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    -c|-S|-E) link_command=false ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -shared|-dynamiclib) static_support_link=false ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$link_command" = true ] && [ "$dynamic_rustc_link" = true ]; then set -- "$@" "$target_runtime_dir/musl-lfs-compat.o" -no-pie -Wl,-Bdynamic "-Wl,-dynamic-linker,$target_runtime_dir/libc.so" -Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; elif [ "$link_command" = true ] && [ "$static_support_link" = true ]; then set -- "$@" "$target_runtime_dir/musl-lfs-compat.o" -static -Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'exec "$target_cc_path" "$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" -D_LARGEFILE64_SOURCE -fno-asynchronous-unwind-tables -B"$target_runtime_dir/" -L"$target_runtime_dir" "$@"' >> "$target_alias_dir/cc"
chmod +x "$target_alias_dir/cc"
if [ -x "$target_tool_dir/$target_cxx_program" ]; then printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/c++"; printf '%s\n' 'set -eu' >> "$target_alias_dir/c++"; printf '%s\n' "MANTLE_TARGET_CC_PATH=\"$target_tool_dir/$target_cxx_program\"" >> "$target_alias_dir/c++"; printf '%s\n' 'export MANTLE_TARGET_CC_PATH' >> "$target_alias_dir/c++"; printf '%s\n' "exec \"$target_alias_dir/cc\" \"\$@\"" >> "$target_alias_dir/c++"; chmod +x "$target_alias_dir/c++"; fi
PATH="$target_alias_dir:$PATH"
export PATH
for target_tool in ar ranlib; do case "$target_tool" in ar) target_program="$target_ar_program" ;; ranlib) target_program="$target_ranlib_program" ;; esac; if [ -x "$target_tool_dir/$target_program" ]; then printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/$target_tool"; printf '%s\n' "exec \"$target_tool_dir/$target_program\" \"\$@\"" >> "$target_alias_dir/$target_tool"; chmod +x "$target_alias_dir/$target_tool"; fi; done
export CC_x86_64_linux_musl="$target_alias_dir/cc"
export CC_x86_64_unknown_linux_musl="$target_alias_dir/cc"
if [ -x "$target_alias_dir/c++" ]; then export CXX_x86_64_unknown_linux_musl="$target_alias_dir/c++"; fi
if [ -x "$target_alias_dir/ar" ]; then export AR_x86_64_unknown_linux_musl="$target_alias_dir/ar"; fi
if [ -x "$target_alias_dir/ranlib" ]; then export RANLIB_x86_64_unknown_linux_musl="$target_alias_dir/ranlib"; fi
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="$target_alias_dir/cc"
printf '%s\n' "using target linker wrapper: $target_alias_dir/cc -> $target_cc_path ($target_cc_machine); runtime CRT/unwind dir: $target_runtime_dir"
else
  printf '%s\n' "using compiler-host linker for $RUSTC_TARGET; target sysroot remains $RUSTC_PROVIDER_TARGET_TRIPLE"
fi
if [ ! -x "$target_alias_dir/cc" ] || [ ! -x "$target_alias_dir/c++" ] || [ ! -x "$target_alias_dir/ar" ] || [ ! -x "$target_alias_dir/ranlib" ]; then printf '%s\n' 'source-root musl LLVM host wrapper tools are incomplete' >&2; exit 3; fi
target_libatomic_source="$target_runtime_dir/mantle-libatomic-shim.c"
target_libatomic_object="$target_runtime_dir/mantle-libatomic-shim.o"
cat > "$target_libatomic_source" <<'MANTLE_MUSL_LIBATOMIC_C'
typedef unsigned char mantle_atomic_u8;

enum {
    ATOMIC_WIDTH_BYTES = sizeof(unsigned __int128),
    ATOMIC_LOCK_FREE = 0,
    ATOMIC_LOCK_HELD = 1,
};

static volatile int mantle_atomic_lock = ATOMIC_LOCK_FREE;

static void mantle_atomic_lock_acquire(void) {
    while (__sync_lock_test_and_set(&mantle_atomic_lock, ATOMIC_LOCK_HELD) != ATOMIC_LOCK_FREE) {
    }
}

static void mantle_atomic_lock_release(void) {
    __sync_lock_release(&mantle_atomic_lock);
}

_Bool __atomic_compare_exchange_16(
    volatile void *mem,
    void *expected,
    unsigned __int128 desired,
    _Bool weak,
    int success_memorder,
    int failure_memorder
) {
    (void)weak;
    (void)success_memorder;
    (void)failure_memorder;

    volatile mantle_atomic_u8 *actual_bytes = (volatile mantle_atomic_u8 *)mem;
    mantle_atomic_u8 *expected_bytes = (mantle_atomic_u8 *)expected;
    mantle_atomic_u8 *desired_bytes = (mantle_atomic_u8 *)&desired;
    mantle_atomic_u8 observed_bytes[ATOMIC_WIDTH_BYTES];
    _Bool matches = 1;

    mantle_atomic_lock_acquire();
    for (unsigned int byte_index = 0; byte_index < ATOMIC_WIDTH_BYTES; byte_index += 1) {
        observed_bytes[byte_index] = actual_bytes[byte_index];
        if (observed_bytes[byte_index] != expected_bytes[byte_index]) {
            matches = 0;
        }
    }
    if (matches) {
        for (unsigned int byte_index = 0; byte_index < ATOMIC_WIDTH_BYTES; byte_index += 1) {
            actual_bytes[byte_index] = desired_bytes[byte_index];
        }
    } else {
        for (unsigned int byte_index = 0; byte_index < ATOMIC_WIDTH_BYTES; byte_index += 1) {
            expected_bytes[byte_index] = observed_bytes[byte_index];
        }
    }
    mantle_atomic_lock_release();
    return matches;
}
MANTLE_MUSL_LIBATOMIC_C
"$target_cc_path" "$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" -fPIC -fno-asynchronous-unwind-tables -c "$target_libatomic_source" -o "$target_libatomic_object"
rm -f "$target_runtime_dir/libatomic.a"; "$target_alias_dir/ar" rcs "$target_runtime_dir/libatomic.a" "$target_libatomic_object"
"$target_alias_dir/ranlib" "$target_runtime_dir/libatomic.a"
MINICARGO_FLAGS="${MINICARGO_FLAGS:-} --target $RUSTC_HOST_TRIPLE"
export MINICARGO_FLAGS
target_static_stdcxx_archive="$target_musl_crt_dir/libstdc++.a"
if [ ! -f "$target_static_stdcxx_archive" ]; then target_static_stdcxx_archive="$target_orig_cc_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libstdc++.a"; fi
if [ -f "$target_static_stdcxx_archive" ]; then
  rm -f "$target_runtime_dir/libstdc++.a"; $COPY_PROGRAM "$target_static_stdcxx_archive" "$target_runtime_dir/libstdc++.a"
  LLVM_STATIC_STDCPP="$target_runtime_dir/libstdc++.a"
else
  if [ ! -f "$target_orig_cc_file" ]; then printf '%s\n' 'source-root musl libstdc++.a missing for LLVM host build' >&2; exit 3; fi
  printf '%s\n' 'source-root musl libstdc++.a unavailable; continuing only for wrapper-backed synthetic route without LLVM_STATIC_STDCPP'
fi
CC_PROGRAM="$target_alias_dir/cc"
CXX_PROGRAM="$target_alias_dir/c++"
AR="$target_alias_dir/ar"
RANLIB="$target_alias_dir/ranlib"
CMAKE_C_COMPILER="$target_alias_dir/cc"
CMAKE_CXX_COMPILER="$target_alias_dir/c++"
CMAKE_AR="$target_alias_dir/ar"
CMAKE_RANLIB="$target_alias_dir/ranlib"
CMAKE_C_COMPILER_AR="$target_alias_dir/ar"
CMAKE_CXX_COMPILER_AR="$target_alias_dir/ar"
CMAKE_C_COMPILER_RANLIB="$target_alias_dir/ranlib"
CMAKE_CXX_COMPILER_RANLIB="$target_alias_dir/ranlib"
MRUSTC_CXXFLAGS="$MRUSTC_CXXFLAGS -static-libstdc++ -static-libgcc"
CC="$CC_PROGRAM"
CXX="$CXX_PROGRAM"
CFLAGS="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_CFLAGS"
CXXFLAGS="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MRUSTC_CXXFLAGS $ZLIB_CFLAGS"
LDFLAGS="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_LIBS"
CPPFLAGS="$ZLIB_CFLAGS"
LIBS="$ZLIB_LIBS"
if [ -n "${LLVM_LINKER_FLAGS:-}" ]; then LLVM_LINKER_FLAGS="-L$target_runtime_dir -lgcc -lunwind $LLVM_LINKER_FLAGS"; else LLVM_LINKER_FLAGS="-L$target_runtime_dir -lgcc -lunwind"; fi
export CC_PROGRAM CXX_PROGRAM CC CXX CFLAGS CPPFLAGS CXXFLAGS LDFLAGS LIBS AR RANLIB CMAKE_C_COMPILER CMAKE_CXX_COMPILER CMAKE_AR CMAKE_RANLIB CMAKE_C_COMPILER_AR CMAKE_CXX_COMPILER_AR CMAKE_C_COMPILER_RANLIB CMAKE_CXX_COMPILER_RANLIB LLVM_STATIC_STDCPP LLVM_LINKER_FLAGS
printf '%s\n' "using source-root musl LLVM host wrappers: CC=$CC_PROGRAM CXX=$CXX_PROGRAM LLVM_STATIC_STDCPP=${LLVM_STATIC_STDCPP:-} LLVM_LINKER_FLAGS=$LLVM_LINKER_FLAGS"
fi
mrustc_patch_file=rustc-${RUSTC_VERSION}-src.patch
mrustc_patch_tmp="$BUILD_DIR/rustc-source-busybox.patch"
mrustc_patch_skip=false
mrustc_patch_skipped=0
: > "$mrustc_patch_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    '--- compiler/rustc_errors/src/lib.rs'|'--- compiler/rustc_middle/src/ty/sty.rs') mrustc_patch_skip=true; mrustc_patch_skipped=$((mrustc_patch_skipped + 1)) ;;
    '--- '*) mrustc_patch_skip=false; printf '%s\n' "$line" >> "$mrustc_patch_tmp" ;;
    *) if [ "$mrustc_patch_skip" = false ]; then printf '%s\n' "$line" >> "$mrustc_patch_tmp"; fi ;;
  esac
done < "$mrustc_patch_file"
if [ "$mrustc_patch_skipped" -ne 2 ]; then printf '%s\n' 'mrustc Rust patch lacks the exact BusyBox-incompatible section set' >&2; exit 3; fi
$COPY_PROGRAM "$mrustc_patch_tmp" "$mrustc_patch_file"
rm -f "$mrustc_patch_tmp"
$MAKE_PROGRAM -j "$PARLEVEL" -f minicargo.mk RUSTCSRC
rustc_errors_presult_source=rustc-${RUSTC_VERSION}-src/compiler/rustc_errors/src/lib.rs
rustc_errors_presult_tmp="$BUILD_DIR/rustc-errors-presult.tmp"
rustc_errors_presult_matches=0
: > "${rustc_errors_presult_tmp}"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = 'rustc_data_structures::static_assert_size!(PResult<'\''_, bool>, 24);' ]; then
    printf '%s\n' '#[cfg(not(rust_compiler="mrustc"))]' >> "${rustc_errors_presult_tmp}"
    printf '%s\n' 'rustc_data_structures::static_assert_size!(PResult<'\''_, bool>, 24);' >> "${rustc_errors_presult_tmp}"
    rustc_errors_presult_matches=$((${rustc_errors_presult_matches} + 1))
  else
    printf '%s\n' "$line" >> "${rustc_errors_presult_tmp}"
  fi
done < "${rustc_errors_presult_source}"
if [ "${rustc_errors_presult_matches}" -ne 1 ]; then printf '%s\n' 'authenticated Rust source rewrite rustc-errors-presult did not match exactly once' >&2; exit 3; fi
$COPY_PROGRAM "${rustc_errors_presult_tmp}" "${rustc_errors_presult_source}"
rm -f "${rustc_errors_presult_tmp}"
rustc_middle_tys_inputs_source=rustc-${RUSTC_VERSION}-src/compiler/rustc_middle/src/ty/sty.rs
rustc_middle_tys_inputs_tmp="$BUILD_DIR/rustc-middle-tys-inputs.tmp"
rustc_middle_tys_inputs_matches=0
: > "${rustc_middle_tys_inputs_tmp}"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = '        self.split_last().unwrap().1' ]; then
    printf '%s\n' '        (**self).split_last().unwrap().1' >> "${rustc_middle_tys_inputs_tmp}"
    rustc_middle_tys_inputs_matches=$((${rustc_middle_tys_inputs_matches} + 1))
  else
    printf '%s\n' "$line" >> "${rustc_middle_tys_inputs_tmp}"
  fi
done < "${rustc_middle_tys_inputs_source}"
if [ "${rustc_middle_tys_inputs_matches}" -ne 1 ]; then printf '%s\n' 'authenticated Rust source rewrite rustc-middle-tys-inputs did not match exactly once' >&2; exit 3; fi
$COPY_PROGRAM "${rustc_middle_tys_inputs_tmp}" "${rustc_middle_tys_inputs_source}"
rm -f "${rustc_middle_tys_inputs_tmp}"
printf '%s\n' 'disabling OpenSSL assembly generators for protected source bootstrap'
set -- rustc-${RUSTC_VERSION}-src/vendor/openssl-src-300.*/src/lib.rs
if [ "$#" -ne 1 ] || [ ! -f "$1" ]; then printf '%s\n' 'vendored OpenSSL source configuration is not unique' >&2; exit 3; fi
openssl_src_config=$1
openssl_src_tmp="$BUILD_DIR/openssl-src-no-asm.rs"
openssl_no_asm_matches=0
: > "$openssl_src_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = '        // On Android it looks like not passing no-stdio may cause a build' ]; then
    printf '%s\n' '        // Mantle: avoid Perl assembly pipes that require ambient /bin/sh.' >> "$openssl_src_tmp"
    printf '%s\n' '        configure.arg("no-asm");' >> "$openssl_src_tmp"
    openssl_no_asm_matches=$((openssl_no_asm_matches + 1))
  fi
  printf '%s\n' "$line" >> "$openssl_src_tmp"
done < "$openssl_src_config"
if [ "$openssl_no_asm_matches" -ne 1 ]; then printf '%s\n' 'vendored OpenSSL configuration lacks one expected insertion point' >&2; exit 3; fi
$COPY_PROGRAM "$openssl_src_tmp" "$openssl_src_config"
rm -f "$openssl_src_tmp"
if [ "$RUSTC_HOST_TRIPLE" = "x86_64-unknown-linux-musl" ]; then
printf '%s\n' 'normalizing Rust explicit sysroot handling for static musl rustc'
rustc_config_source=rustc-${RUSTC_VERSION}-src/compiler/rustc_session/src/config.rs
if [ ! -f "$rustc_config_source" ]; then printf '%s\n' 'rustc_session config.rs missing before sysroot normalization' >&2; exit 3; fi
rustc_config_tmp="$BUILD_DIR/rustc-session-config-sysroot.rs"
rustc_sysroot_original_line='        Sysroot { explicit, default: filesearch::default_sysroot() }'
rustc_sysroot_replacement_line_1='        match explicit {'
rustc_sysroot_replacement_line_2='            Some(explicit) => Sysroot { default: explicit.clone(), explicit: Some(explicit) },'
rustc_sysroot_replacement_line_3='            None => Sysroot { explicit: None, default: filesearch::default_sysroot() },'
rustc_sysroot_replacement_line_4='        }'
rustc_sysroot_replaced=false
rustc_sysroot_seen=false
: > "$rustc_config_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = "$rustc_sysroot_original_line" ]; then
    printf '%s\n' "$rustc_sysroot_replacement_line_1" >> "$rustc_config_tmp"
    printf '%s\n' "$rustc_sysroot_replacement_line_2" >> "$rustc_config_tmp"
    printf '%s\n' "$rustc_sysroot_replacement_line_3" >> "$rustc_config_tmp"
    printf '%s\n' "$rustc_sysroot_replacement_line_4" >> "$rustc_config_tmp"
    rustc_sysroot_replaced=true
  else
    printf '%s\n' "$line" >> "$rustc_config_tmp"
    if [ "$line" = "$rustc_sysroot_replacement_line_1" ]; then rustc_sysroot_seen=true; fi
  fi
done < "$rustc_config_source"
if [ "$rustc_sysroot_replaced" = false ] && [ "$rustc_sysroot_seen" = false ]; then printf '%s\n' 'rustc_session Sysroot::new no longer has expected default_sysroot shape' >&2; exit 3; fi
$COPY_PROGRAM "$rustc_config_tmp" "$rustc_config_source"
rm -f "$rustc_config_tmp"
fi
$MAKE_PROGRAM -j "$PARLEVEL" -f minicargo.mk output/rustc
$MAKE_PROGRAM -j "$PARLEVEL" -f minicargo.mk output/cargo
if [ "$RUSTC_HOST_TRIPLE" = "x86_64-unknown-linux-musl" ]; then
printf '%s\n' 'normalizing rustc_driver crate type for static musl compiler host'
rustc_driver_manifest=rustc-${RUSTC_VERSION}-src/compiler/rustc_driver/Cargo.toml
if [ ! -f "$rustc_driver_manifest" ]; then printf '%s\n' 'rustc_driver manifest missing before musl host normalization' >&2; exit 3; fi
rustc_driver_manifest_tmp="$BUILD_DIR/rustc-driver-Cargo.toml"
rustc_driver_replaced=false
rustc_driver_seen_rlib=false
: > "$rustc_driver_manifest_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    'crate-type = ["dylib"]') printf '%s\n' 'crate-type = ["rlib"]' >> "$rustc_driver_manifest_tmp"; rustc_driver_replaced=true ;;
    'crate-type = ["rlib"]') printf '%s\n' "$line" >> "$rustc_driver_manifest_tmp"; rustc_driver_seen_rlib=true ;;
    *) printf '%s\n' "$line" >> "$rustc_driver_manifest_tmp" ;;
  esac
done < "$rustc_driver_manifest"
if [ "$rustc_driver_replaced" = false ] && [ "$rustc_driver_seen_rlib" = false ]; then printf '%s\n' 'rustc_driver manifest lacks expected crate-type line for musl host normalization' >&2; exit 3; fi
$COPY_PROGRAM "$rustc_driver_manifest_tmp" "$rustc_driver_manifest"
rm -f "$rustc_driver_manifest_tmp"
fi
RUSTC_TARGET="$RUSTC_HOST_TRIPLE"
if [ "$RUSTC_TARGET" = "$RUSTC_PROVIDER_TARGET_TRIPLE" ] && [ "$RUSTC_TARGET" = "x86_64-unknown-linux-musl" ]; then
if [ ! -x "$TARGET_CC_PROGRAM" ]; then printf '%s\n' 'receipt-bound x86_64 musl target gcc is missing' >&2; exit 3; fi
target_cc_path="$TARGET_CC_PROGRAM"
target_toolchain_root="$MANTLE_TARGET_TOOLCHAIN_ROOT"
target_cc_machine=$("$target_cc_path" -dumpmachine 2>/dev/null || true)
case " $TARGET_MUSL_MACHINE_ALIASES " in *" $target_cc_machine "*) ;; *) printf '%s\n' "target gcc machine expected one of: $TARGET_MUSL_MACHINE_ALIASES; got $target_cc_machine" >&2; exit 3 ;; esac
target_tool_dir=$(dirname "$target_cc_path")
target_wrapper_root=$(dirname "$target_tool_dir")
target_cc_basename=${target_cc_path##*/}
case "$target_cc_basename" in *-gcc) target_tool_prefix=${target_cc_basename%-gcc} ;; *) target_tool_prefix= ;; esac
if [ -z "$target_tool_prefix" ]; then printf '%s\n' "target gcc name does not end in -gcc: $target_cc_basename" >&2; exit 3; fi
target_cxx_program=$target_tool_prefix-g++
target_ar_program=$target_tool_prefix-ar
target_ranlib_program=$target_tool_prefix-ranlib
target_objcopy_program=$target_tool_prefix-objcopy
if [ -n "$target_toolchain_root" ]; then
  for target_required_tool in "$target_tool_dir/$target_cxx_program" "$target_tool_dir/$target_ar_program" "$target_tool_dir/$target_ranlib_program" "$target_tool_dir/$target_objcopy_program"; do
    if [ ! -x "$target_required_tool" ]; then printf '%s\n' "source-root musl target toolchain root is incomplete: $target_required_tool" >&2; exit 3; fi
  done
  for target_required_file in "$target_toolchain_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libc.so" "$target_toolchain_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libgcc_s.so.1"; do
    if [ ! -f "$target_required_file" ]; then printf '%s\n' "source-root musl target toolchain root is incomplete: $target_required_file" >&2; exit 3; fi
  done
fi
target_nix_support="$target_wrapper_root/nix-support"
target_orig_libc_file="$target_nix_support/orig-libc"
target_orig_cc_file="$target_nix_support/orig-cc"
if [ -f "$target_orig_libc_file" ] && [ -f "$target_orig_cc_file" ]; then
  target_libc_root=$(cat "$target_orig_libc_file")
  target_orig_cc_root=$(cat "$target_orig_cc_file")
  target_gcc_crt_machine=$RUSTC_TARGET
else
  target_libc_root="$target_wrapper_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT"
  target_orig_cc_root="$target_wrapper_root"
  target_gcc_crt_machine=$target_cc_machine
fi
target_musl_crt_dir="$target_libc_root/lib"
target_gcc_version=$("$target_cc_path" -dumpfullversion 2>/dev/null || "$target_cc_path" -dumpversion 2>/dev/null || true)
target_gcc_crt_dir="$target_orig_cc_root/lib/gcc/$target_gcc_crt_machine/$target_gcc_version"
if [ ! -f "$target_gcc_crt_dir/crtbeginS.o" ]; then for candidate_dir in "$target_orig_cc_root"/lib/gcc/"$target_gcc_crt_machine"/*; do if [ -f "$candidate_dir/crtbeginS.o" ]; then target_gcc_crt_dir="$candidate_dir"; break; fi; done; fi
if [ ! -f "$target_musl_crt_dir/crt1.o" ] || [ ! -f "$target_musl_crt_dir/Scrt1.o" ] || [ ! -f "$target_musl_crt_dir/rcrt1.o" ] || [ ! -f "$target_musl_crt_dir/crti.o" ] || [ ! -f "$target_musl_crt_dir/crtn.o" ] || [ ! -f "$target_gcc_crt_dir/crtbeginS.o" ] || [ ! -f "$target_gcc_crt_dir/libgcc.a" ]; then printf '%s\n' 'target gcc toolchain does not expose musl/gcc CRT and unwinder objects' >&2; exit 3; fi
export NIX_CC_WRAPPER_TARGET_HOST_x86_64_unknown_linux_musl=1
export COPY_PROGRAM
target_alias_dir="$BUILD_DIR/target-linker-bin"
target_runtime_dir="$BUILD_DIR/target-linker-runtime"
mkdir -p "$target_alias_dir" "$target_runtime_dir"
for crt_name in crt1.o Scrt1.o rcrt1.o crti.o crtn.o; do rm -f "$target_runtime_dir/$crt_name"; $COPY_PROGRAM "$target_musl_crt_dir/$crt_name" "$target_runtime_dir/$crt_name"; done
for crt_name in crtbeginS.o crtendS.o; do rm -f "$target_runtime_dir/$crt_name"; $COPY_PROGRAM "$target_gcc_crt_dir/$crt_name" "$target_runtime_dir/$crt_name"; done
if [ ! -x "$target_tool_dir/$target_objcopy_program" ]; then printf '%s\n' 'target gcc toolchain does not expose objcopy for CRT normalization' >&2; exit 3; fi
target_crtbegin_no_frame_init="$target_runtime_dir/crtbeginS.o.no-frame-init"
rm -f "$target_crtbegin_no_frame_init"
"$target_tool_dir/$target_objcopy_program" --remove-section .init_array --remove-section .rela.init_array --remove-section .fini_array --remove-section .rela.fini_array "$target_runtime_dir/crtbeginS.o" "$target_crtbegin_no_frame_init"
rm -f "$target_runtime_dir/crtbeginS.o"
$COPY_PROGRAM "$target_crtbegin_no_frame_init" "$target_runtime_dir/crtbeginS.o"
rm -f "$target_crtbegin_no_frame_init"
target_unwind_archive="$target_gcc_crt_dir/libgcc_eh.a"
if [ ! -f "$target_unwind_archive" ]; then target_unwind_archive="$target_gcc_crt_dir/libgcc.a"; fi
rm -f "$target_runtime_dir/libunwind.a"; $COPY_PROGRAM "$target_unwind_archive" "$target_runtime_dir/libunwind.a"
rm -f "$target_runtime_dir/libgcc.a"; $COPY_PROGRAM "$target_gcc_crt_dir/libgcc.a" "$target_runtime_dir/libgcc.a"
rm -f "$target_runtime_dir/libgcc_s.a" "$target_runtime_dir/libgcc_s.so" "$target_runtime_dir/libgcc_s.so.1"
$COPY_PROGRAM "$target_gcc_crt_dir/libgcc.a" "$target_runtime_dir/libgcc_s.a"
for target_libgcc_shared_name in libgcc_s.so libgcc_s.so.1; do
  for target_libgcc_shared_dir in "$target_musl_crt_dir" "$target_orig_cc_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib" "$target_gcc_crt_dir"; do
    if [ -f "$target_libgcc_shared_dir/$target_libgcc_shared_name" ]; then $COPY_PROGRAM "$target_libgcc_shared_dir/$target_libgcc_shared_name" "$target_runtime_dir/$target_libgcc_shared_name"; break; fi
  done
done
target_lfs_compat_source="$target_runtime_dir/musl-lfs-compat.c"
target_lfs_compat_object="$target_runtime_dir/musl-lfs-compat.o"
cat > "$target_lfs_compat_source" <<'MANTLE_MUSL_LFS_COMPAT_C'
#define _LARGEFILE64_SOURCE 1
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <sys/mman.h>
#include <sys/sendfile.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#ifdef fstat64
#undef fstat64
#endif
#ifdef fstatat64
#undef fstatat64
#endif
#ifdef ftruncate64
#undef ftruncate64
#endif
#ifdef sendfile64
#undef sendfile64
#endif
#ifdef lseek64
#undef lseek64
#endif
#ifdef lstat64
#undef lstat64
#endif
#ifdef mmap64
#undef mmap64
#endif
#ifdef open64
#undef open64
#endif
#ifdef openat64
#undef openat64
#endif
#ifdef pread64
#undef pread64
#endif
#ifdef pwrite64
#undef pwrite64
#endif
#ifdef readdir64
#undef readdir64
#endif
#ifdef stat64
#undef stat64
#endif

#define MANTLE_PTHREAD_TLS_KEY_CAPACITY 4096u

typedef void (*mantle_pthread_tls_destructor)(void *);

static int mantle_pthread_tls_used[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static mantle_pthread_tls_destructor mantle_pthread_tls_destructors[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static __thread void *mantle_pthread_tls_values[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static unsigned int mantle_pthread_tls_next_key;

static int mantle_pthread_tls_key_is_valid(pthread_key_t key) {
    unsigned int index = (unsigned int)key;
    if (index >= MANTLE_PTHREAD_TLS_KEY_CAPACITY) {
        return 0;
    }
    if (__sync_add_and_fetch(&mantle_pthread_tls_used[index], 0) == 0) {
        return 0;
    }
    return 1;
}

int __wrap_pthread_key_create(pthread_key_t *key, void (*destructor)(void *)) {
    if (key == NULL) {
        return EINVAL;
    }
    for (unsigned int offset = 0; offset < MANTLE_PTHREAD_TLS_KEY_CAPACITY; offset++) {
        unsigned int candidate = (mantle_pthread_tls_next_key + offset) % MANTLE_PTHREAD_TLS_KEY_CAPACITY;
        if (__sync_bool_compare_and_swap(&mantle_pthread_tls_used[candidate], 0, 1) != 0) {
            mantle_pthread_tls_destructors[candidate] = destructor;
            *key = (pthread_key_t)candidate;
            mantle_pthread_tls_next_key = (candidate + 1) % MANTLE_PTHREAD_TLS_KEY_CAPACITY;
            return 0;
        }
    }
    return EAGAIN;
}

int __wrap_pthread_key_delete(pthread_key_t key) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return EINVAL;
    }
    unsigned int index = (unsigned int)key;
    mantle_pthread_tls_values[index] = NULL;
    mantle_pthread_tls_destructors[index] = NULL;
    __sync_lock_release(&mantle_pthread_tls_used[index]);
    return 0;
}

void *__wrap_pthread_getspecific(pthread_key_t key) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return NULL;
    }
    return mantle_pthread_tls_values[(unsigned int)key];
}

int __wrap_pthread_setspecific(pthread_key_t key, const void *value) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return EINVAL;
    }
    mantle_pthread_tls_values[(unsigned int)key] = (void *)value;
    return 0;
}

int backtrace(void **buffer, int size) {
    (void)buffer;
    (void)size;
    return 0;
}

void backtrace_symbols_fd(void *const *buffer, int size, int fd) {
    (void)buffer;
    (void)size;
    (void)fd;
}

static int mantle_open_flags_need_mode(int flags) {
    if ((flags & O_CREAT) != 0) {
        return 1;
    }
#ifdef O_TMPFILE
    if ((flags & O_TMPFILE) == O_TMPFILE) {
        return 1;
    }
#endif
    return 0;
}

int fstat64(int fd, struct stat *buf) {
    return fstat(fd, buf);
}

int fstatat64(int dirfd, const char *pathname, struct stat *buf, int flags) {
    return fstatat(dirfd, pathname, buf, flags);
}

int ftruncate64(int fd, off_t length) {
    return ftruncate(fd, length);
}

ssize_t sendfile64(int out_fd, int in_fd, off_t *offset, size_t count) {
    return sendfile(out_fd, in_fd, offset, count);
}

off_t lseek64(int fd, off_t offset, int whence) {
    return lseek(fd, offset, whence);
}

int lstat64(const char *pathname, struct stat *buf) {
    return lstat(pathname, buf);
}

void *mmap64(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    return mmap(addr, length, prot, flags, fd, offset);
}

int open64(const char *pathname, int flags, ...) {
    mode_t mode = 0;
    if (mantle_open_flags_need_mode(flags) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
        return open(pathname, flags, mode);
    }
    return open(pathname, flags);
}

int openat64(int dirfd, const char *pathname, int flags, ...) {
    mode_t mode = 0;
    if (mantle_open_flags_need_mode(flags) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
        return openat(dirfd, pathname, flags, mode);
    }
    return openat(dirfd, pathname, flags);
}

ssize_t pread64(int fd, void *buf, size_t count, off_t offset) {
    return pread(fd, buf, count, offset);
}

ssize_t pwrite64(int fd, const void *buf, size_t count, off_t offset) {
    return pwrite(fd, buf, count, offset);
}

struct dirent *readdir64(DIR *dirp) {
    return readdir(dirp);
}

int stat64(const char *pathname, struct stat *buf) {
    return stat(pathname, buf);
}
MANTLE_MUSL_LFS_COMPAT_C
"$target_cc_path" "$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" -D_LARGEFILE64_SOURCE -fno-asynchronous-unwind-tables -fPIC -c "$target_lfs_compat_source" -o "$target_lfs_compat_object"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/sh"
printf '%s\n' "exec \"$SHELL_PROGRAM\" \"\$@\"" >> "$target_alias_dir/sh"
chmod +x "$target_alias_dir/sh"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/emcc"
printf '%s\n' 'exit 127' >> "$target_alias_dir/emcc"
chmod +x "$target_alias_dir/emcc"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/pkg-config"
chmod +x "$target_alias_dir/pkg-config"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/pkgconf"
chmod +x "$target_alias_dir/pkgconf"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/git"
chmod +x "$target_alias_dir/git"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/perl"
printf '%s\n' "exec \"$PERL_PROGRAM\" \"\$@\"" >> "$target_alias_dir/perl"
chmod +x "$target_alias_dir/perl"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/make"
printf '%s\n' "exec \"$MAKE_PROGRAM\" \"\$@\"" >> "$target_alias_dir/make"
chmod +x "$target_alias_dir/make"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/cc"
printf '%s\n' 'set -eu' >> "$target_alias_dir/cc"
printf '%s\n' "target_cc_path=\"\${MANTLE_TARGET_CC_PATH:-$target_cc_path}\"" >> "$target_alias_dir/cc"
printf '%s\n' "target_runtime_dir=\"$target_runtime_dir\"" >> "$target_alias_dir/cc"
printf '%s\n' 'static_pie_normalized=false' >> "$target_alias_dir/cc"
printf '%s\n' 'output_path=' >> "$target_alias_dir/cc"
printf '%s\n' 'previous_arg=' >> "$target_alias_dir/cc"
printf '%s\n' 'rustc_main_response_file=false' >> "$target_alias_dir/cc"
printf '%s\n' 'shared_link=false' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  if [ "$previous_arg" = "-o" ]; then output_path="$arg"; previous_arg=; continue; fi' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    -static-pie) static_pie_normalized=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -shared|-dynamiclib) shared_link=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -o) previous_arg=-o ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    @*/output/rustc-build/rustc_main_cmd.txt|@output/rustc-build/rustc_main_cmd.txt) rustc_main_response_file=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'dynamic_rustc_link=false' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$rustc_main_response_file" = true ]; then dynamic_rustc_link=true; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'case "$output_path" in' >> "$target_alias_dir/cc"
printf '%s\n' '  */output/rustc|output/rustc|*/output/rustc-build/rustc_main|output/rustc-build/rustc_main) dynamic_rustc_link=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' 'esac' >> "$target_alias_dir/cc"
printf '%s\n' 'mapped_args_set=false' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    rcrt1.o|*/rcrt1.o) mapped_arg="$target_runtime_dir/crt1.o" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    crt1.o|*/crt1.o|Scrt1.o|*/Scrt1.o|crti.o|*/crti.o|crtn.o|*/crtn.o|crtbeginS.o|*/crtbeginS.o|crtendS.o|*/crtendS.o) mapped_arg="$target_runtime_dir/${arg##*/}" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -static) if [ "$shared_link" = true ]; then continue; else mapped_arg="$arg"; fi ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -static-pie) if [ "$dynamic_rustc_link" = true ] || [ "$shared_link" = true ]; then continue; else mapped_arg="-static"; fi ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    *) mapped_arg="$arg" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' '  if [ "$mapped_args_set" = false ]; then set -- "$mapped_arg"; mapped_args_set=true; else set -- "$@" "$mapped_arg"; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$mapped_args_set" = false ]; then set --; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'link_command=true' >> "$target_alias_dir/cc"
printf '%s\n' 'static_support_link=true' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    -c|-S|-E) link_command=false ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -shared|-dynamiclib) static_support_link=false ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$link_command" = true ] && [ "$dynamic_rustc_link" = true ]; then set -- "$@" "$target_runtime_dir/musl-lfs-compat.o" -no-pie -Wl,-Bdynamic "-Wl,-dynamic-linker,$target_runtime_dir/libc.so" -Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; elif [ "$link_command" = true ] && [ "$static_support_link" = true ]; then set -- "$@" "$target_runtime_dir/musl-lfs-compat.o" -static -Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'exec "$target_cc_path" "$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" -D_LARGEFILE64_SOURCE -fno-asynchronous-unwind-tables -B"$target_runtime_dir/" -L"$target_runtime_dir" "$@"' >> "$target_alias_dir/cc"
chmod +x "$target_alias_dir/cc"
if [ -x "$target_tool_dir/$target_cxx_program" ]; then printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/c++"; printf '%s\n' 'set -eu' >> "$target_alias_dir/c++"; printf '%s\n' "MANTLE_TARGET_CC_PATH=\"$target_tool_dir/$target_cxx_program\"" >> "$target_alias_dir/c++"; printf '%s\n' 'export MANTLE_TARGET_CC_PATH' >> "$target_alias_dir/c++"; printf '%s\n' "exec \"$target_alias_dir/cc\" \"\$@\"" >> "$target_alias_dir/c++"; chmod +x "$target_alias_dir/c++"; fi
PATH="$target_alias_dir:$PATH"
export PATH
for target_tool in ar ranlib; do case "$target_tool" in ar) target_program="$target_ar_program" ;; ranlib) target_program="$target_ranlib_program" ;; esac; if [ -x "$target_tool_dir/$target_program" ]; then printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/$target_tool"; printf '%s\n' "exec \"$target_tool_dir/$target_program\" \"\$@\"" >> "$target_alias_dir/$target_tool"; chmod +x "$target_alias_dir/$target_tool"; fi; done
export CC_x86_64_linux_musl="$target_alias_dir/cc"
export CC_x86_64_unknown_linux_musl="$target_alias_dir/cc"
if [ -x "$target_alias_dir/c++" ]; then export CXX_x86_64_unknown_linux_musl="$target_alias_dir/c++"; fi
if [ -x "$target_alias_dir/ar" ]; then export AR_x86_64_unknown_linux_musl="$target_alias_dir/ar"; fi
if [ -x "$target_alias_dir/ranlib" ]; then export RANLIB_x86_64_unknown_linux_musl="$target_alias_dir/ranlib"; fi
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="$target_alias_dir/cc"
printf '%s\n' "using target linker wrapper: $target_alias_dir/cc -> $target_cc_path ($target_cc_machine); runtime CRT/unwind dir: $target_runtime_dir"
else
  printf '%s\n' "using compiler-host linker for $RUSTC_TARGET; target sysroot remains $RUSTC_PROVIDER_TARGET_TRIPLE"
fi
if [ "$RUSTC_TARGET" = "x86_64-unknown-linux-musl" ] && [ "$RUSTC_PROVIDER_TARGET_TRIPLE" = "x86_64-unknown-linux-musl" ]; then
printf '%s\n' 'preparing source-root musl proc-macro runtime search path'
target_musl_shared_libc="$target_musl_crt_dir/libc.so"
if [ ! -f "$target_musl_shared_libc" ]; then printf '%s\n' 'source-root musl libc.so missing for proc-macro runtime' >&2; exit 3; fi
rm -f "$target_runtime_dir/libc.so"; $COPY_PROGRAM "$target_musl_shared_libc" "$target_runtime_dir/libc.so"
run_rustc_makefile=run_rustc/Makefile
if [ ! -f "$run_rustc_makefile" ]; then printf '%s\n' 'mrustc run_rustc Makefile missing before proc-macro runtime normalization' >&2; exit 3; fi
run_rustc_ld_tmp="$BUILD_DIR/run-rustc-Makefile"
run_rustc_ld_original_line='RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))'
run_rustc_ld_runtime_line="RUSTC_ENV_VARS += LD_LIBRARY_PATH=$target_runtime_dir:\$(abspath \$(PREFIX_2)lib):\$(abspath \$(LIBDIR))"
run_rustc_stage2_original_line='CARGO_ENV_STAGE2_STD := CARGO_TARGET_DIR=$(OUTDIR)build-std2 RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)'
run_rustc_stage2_runtime_line='CARGO_ENV_STAGE2_STD := $(RUSTC_ENV_VARS) CARGO_TARGET_DIR=$(OUTDIR)build-std2 RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)'
run_rustc_final_original_line='CARGO_ENV_RUSTC := CARGO_TARGET_DIR=$(OUTDIR)build-rustc RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)'
run_rustc_final_prefix2_line='CARGO_ENV_RUSTC := CARGO_TARGET_DIR=$(OUTDIR)build-rustc RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_2)rustc) $(CARGO_ENV)'
run_rustc_ld_replaced=false
run_rustc_ld_seen_runtime=false
run_rustc_stage2_replaced=false
run_rustc_stage2_seen_runtime=false
run_rustc_final_replaced=false
run_rustc_final_seen_prefix2=false
: > "$run_rustc_ld_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = "$run_rustc_ld_original_line" ]; then
    printf '%s\n' "$run_rustc_ld_runtime_line" >> "$run_rustc_ld_tmp"
    run_rustc_ld_replaced=true
  elif [ "$line" = "$run_rustc_ld_runtime_line" ]; then
    printf '%s\n' "$line" >> "$run_rustc_ld_tmp"
    run_rustc_ld_seen_runtime=true
  elif [ "$line" = "$run_rustc_stage2_original_line" ]; then
    printf '%s\n' "$run_rustc_stage2_runtime_line" >> "$run_rustc_ld_tmp"
    run_rustc_stage2_replaced=true
  elif [ "$line" = "$run_rustc_stage2_runtime_line" ]; then
    printf '%s\n' "$line" >> "$run_rustc_ld_tmp"
    run_rustc_stage2_seen_runtime=true
  elif [ "$line" = "$run_rustc_final_original_line" ]; then
    printf '%s\n' "$run_rustc_final_prefix2_line" >> "$run_rustc_ld_tmp"
    run_rustc_final_replaced=true
  elif [ "$line" = "$run_rustc_final_prefix2_line" ]; then
    printf '%s\n' "$line" >> "$run_rustc_ld_tmp"
    run_rustc_final_seen_prefix2=true
  else
    printf '%s\n' "$line" >> "$run_rustc_ld_tmp"
  fi
done < "$run_rustc_makefile"
if [ "$run_rustc_ld_replaced" = false ] && [ "$run_rustc_ld_seen_runtime" = false ]; then printf '%s\n' 'mrustc run_rustc Makefile lacks expected LD_LIBRARY_PATH line for proc-macro runtime normalization' >&2; exit 3; fi
if [ "$run_rustc_stage2_replaced" = false ] && [ "$run_rustc_stage2_seen_runtime" = false ]; then printf '%s\n' 'mrustc run_rustc Makefile lacks expected stage2 Cargo env line for runtime normalization' >&2; exit 3; fi
if [ "$run_rustc_final_replaced" = false ] && [ "$run_rustc_final_seen_prefix2" = false ]; then printf '%s\n' 'mrustc run_rustc Makefile lacks expected final rustc Cargo env line for prefix-2 host normalization' >&2; exit 3; fi
$COPY_PROGRAM "$run_rustc_ld_tmp" "$run_rustc_makefile"
rm -f "$run_rustc_ld_tmp"
printf '%s\n' 'normalizing run_rustc static rustc sysroot wrappers'
run_rustc_sysroot_tmp="$BUILD_DIR/run-rustc-sysroot-Makefile"
run_rustc_stage1_rule='$(BINDIR_S)rustc: ../output$(OUTDIR_SUF)/rustc'
run_rustc_stage2_rule='$(BINDIR_2)rustc: ../output$(OUTDIR_SUF)/rustc'
run_rustc_copy_line='	cp $< $@'
run_rustc_stage_symlink_line='	ln -sf "$(abspath $<)" "$@.bin"'
run_rustc_stage1_wrapper_line="	printf '#!/bin/sh\nLD_LIBRARY_PATH=\"$target_runtime_dir:\$(abspath \$(PREFIX_2)lib):\$(abspath \$(LIBDIR))\" \"$target_runtime_dir/libc.so\" \"\$\$0.bin\" --sysroot \"\$(abspath \$(PREFIX_S))\" \"\$\$@\"\n' >\$@"
run_rustc_stage2_wrapper_line="	printf '#!/bin/sh\nLD_LIBRARY_PATH=\"$target_runtime_dir:\$(abspath \$(PREFIX_2)lib):\$(abspath \$(LIBDIR))\" \"$target_runtime_dir/libc.so\" \"\$\$0.bin\" --sysroot \"\$(abspath \$(PREFIX_2))\" \"\$\$@\"\n' >\$@"
run_rustc_chmod_line='	chmod +x $@'
run_rustc_final_wrapper_line='	$Vprintf '\''#!/bin/sh\nd=$$(dirname $$0)\nLD_LIBRARY_PATH="$(abspath $(OUTDIR)prefix/lib):$(abspath $(LIBDIR))" $$d/rustc_binary "$$@"'\'' >$@'
run_rustc_final_symlink_line='	$Vln -sf rustc_binary $(BINDIR)rustc_binary.sysroot'
run_rustc_final_sysroot_wrapper_line='	$Vprintf '\''#!/bin/sh\nd=$$(dirname $$0)\nLD_LIBRARY_PATH="$(abspath $(OUTDIR)prefix/lib):$(abspath $(LIBDIR))" $$d/rustc_binary.sysroot --sysroot "$$d/.." "$$@"'\'' >$@'
run_rustc_rule_context=
run_rustc_stage1_wrapper_replaced=false
run_rustc_stage1_wrapper_seen=false
run_rustc_stage2_wrapper_replaced=false
run_rustc_stage2_wrapper_seen=false
run_rustc_final_symlink_seen=false
run_rustc_final_wrapper_replaced=false
run_rustc_final_wrapper_seen=false
: > "$run_rustc_sysroot_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = "$run_rustc_stage1_rule" ]; then
    run_rustc_rule_context=stage1
    printf '%s\n' "$line" >> "$run_rustc_sysroot_tmp"
  elif [ "$line" = "$run_rustc_stage2_rule" ]; then
    run_rustc_rule_context=stage2
    printf '%s\n' "$line" >> "$run_rustc_sysroot_tmp"
  elif [ "$line" = "$run_rustc_copy_line" ] && [ "$run_rustc_rule_context" = stage1 ]; then
    printf '%s\n' "$run_rustc_stage_symlink_line" >> "$run_rustc_sysroot_tmp"
    printf '%s\n' "$run_rustc_stage1_wrapper_line" >> "$run_rustc_sysroot_tmp"
    printf '%s\n' "$run_rustc_chmod_line" >> "$run_rustc_sysroot_tmp"
    run_rustc_stage1_wrapper_replaced=true
    run_rustc_rule_context=
  elif [ "$line" = "$run_rustc_copy_line" ] && [ "$run_rustc_rule_context" = stage2 ]; then
    printf '%s\n' "$run_rustc_stage_symlink_line" >> "$run_rustc_sysroot_tmp"
    printf '%s\n' "$run_rustc_stage2_wrapper_line" >> "$run_rustc_sysroot_tmp"
    printf '%s\n' "$run_rustc_chmod_line" >> "$run_rustc_sysroot_tmp"
    run_rustc_stage2_wrapper_replaced=true
    run_rustc_rule_context=
  elif [ "$line" = "$run_rustc_stage1_wrapper_line" ]; then
    printf '%s\n' "$line" >> "$run_rustc_sysroot_tmp"
    run_rustc_stage1_wrapper_seen=true
    run_rustc_rule_context=
  elif [ "$line" = "$run_rustc_stage2_wrapper_line" ]; then
    printf '%s\n' "$line" >> "$run_rustc_sysroot_tmp"
    run_rustc_stage2_wrapper_seen=true
    run_rustc_rule_context=
  elif [ "$line" = "$run_rustc_final_wrapper_line" ]; then
    printf '%s\n' "$run_rustc_final_symlink_line" >> "$run_rustc_sysroot_tmp"
    printf '%s\n' "$run_rustc_final_sysroot_wrapper_line" >> "$run_rustc_sysroot_tmp"
    run_rustc_final_symlink_seen=true
    run_rustc_final_wrapper_replaced=true
    run_rustc_rule_context=
  elif [ "$line" = "$run_rustc_final_symlink_line" ]; then
    printf '%s\n' "$line" >> "$run_rustc_sysroot_tmp"
    run_rustc_final_symlink_seen=true
    run_rustc_rule_context=
  elif [ "$line" = "$run_rustc_final_sysroot_wrapper_line" ]; then
    printf '%s\n' "$line" >> "$run_rustc_sysroot_tmp"
    run_rustc_final_wrapper_seen=true
    run_rustc_rule_context=
  else
    printf '%s\n' "$line" >> "$run_rustc_sysroot_tmp"
  fi
done < "$run_rustc_makefile"
if [ "$run_rustc_stage1_wrapper_replaced" = false ] && [ "$run_rustc_stage1_wrapper_seen" = false ]; then printf '%s\n' 'mrustc run_rustc Makefile lacks expected stage1 rustc copy rule for sysroot normalization' >&2; exit 3; fi
if [ "$run_rustc_stage2_wrapper_replaced" = false ] && [ "$run_rustc_stage2_wrapper_seen" = false ]; then printf '%s\n' 'mrustc run_rustc Makefile lacks expected stage2 rustc copy rule for sysroot normalization' >&2; exit 3; fi
if [ "$run_rustc_final_symlink_seen" = false ]; then printf '%s\n' 'mrustc run_rustc Makefile lacks expected final rustc symlink rule for sysroot normalization' >&2; exit 3; fi
if [ "$run_rustc_final_wrapper_replaced" = false ] && [ "$run_rustc_final_wrapper_seen" = false ]; then printf '%s\n' 'mrustc run_rustc Makefile lacks expected final rustc wrapper for sysroot normalization' >&2; exit 3; fi
$COPY_PROGRAM "$run_rustc_sysroot_tmp" "$run_rustc_makefile"
rm -f "$run_rustc_sysroot_tmp"
printf '%s\n' 'normalizing run_rustc Cargo static feature set for source-root musl host'
run_rustc_cargo_tmp="$BUILD_DIR/run-rustc-cargo-all-static-Makefile"
run_rustc_cargo_original_line='	$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(BINDIR_S)cargo build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml'
run_rustc_cargo_patched_line='	$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(BINDIR_S)cargo build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml --features all-static'
run_rustc_cargo_replaced=false
run_rustc_cargo_seen_patched=false
: > "$run_rustc_cargo_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  if [ "$line" = "$run_rustc_cargo_original_line" ]; then
    printf '%s\n' "$run_rustc_cargo_patched_line" >> "$run_rustc_cargo_tmp"
    run_rustc_cargo_replaced=true
  elif [ "$line" = "$run_rustc_cargo_patched_line" ]; then
    printf '%s\n' "$line" >> "$run_rustc_cargo_tmp"
    run_rustc_cargo_seen_patched=true
  else
    printf '%s\n' "$line" >> "$run_rustc_cargo_tmp"
  fi
done < "$run_rustc_makefile"
if [ "$run_rustc_cargo_replaced" = false ] && [ "$run_rustc_cargo_seen_patched" = false ]; then printf '%s\n' 'mrustc run_rustc Makefile lacks expected Cargo build line for all-static normalization' >&2; exit 3; fi
$COPY_PROGRAM "$run_rustc_cargo_tmp" "$run_rustc_makefile"
rm -f "$run_rustc_cargo_tmp"
fi
if [ "$RUSTC_TARGET" = "x86_64-unknown-linux-musl" ]; then RUN_RUSTC_DYLIB_EXT=rlib; else RUN_RUSTC_DYLIB_EXT=; fi
if [ -n "$RUN_RUSTC_DYLIB_EXT" ]; then $MAKE_PROGRAM -j "$PARLEVEL" -C run_rustc DYLIB_EXT="$RUN_RUSTC_DYLIB_EXT"; else $MAKE_PROGRAM -j "$PARLEVEL" -C run_rustc; fi
if [ "$RUSTC_PROVIDER_TARGET_TRIPLE" != "$RUSTC_HOST_TRIPLE" ]; then
RUSTC_TARGET="$RUSTC_PROVIDER_TARGET_TRIPLE"
if [ "$RUSTC_TARGET" = "$RUSTC_PROVIDER_TARGET_TRIPLE" ] && [ "$RUSTC_TARGET" = "x86_64-unknown-linux-musl" ]; then
if [ ! -x "$TARGET_CC_PROGRAM" ]; then printf '%s\n' 'receipt-bound x86_64 musl target gcc is missing' >&2; exit 3; fi
target_cc_path="$TARGET_CC_PROGRAM"
target_toolchain_root="$MANTLE_TARGET_TOOLCHAIN_ROOT"
target_cc_machine=$("$target_cc_path" -dumpmachine 2>/dev/null || true)
case " $TARGET_MUSL_MACHINE_ALIASES " in *" $target_cc_machine "*) ;; *) printf '%s\n' "target gcc machine expected one of: $TARGET_MUSL_MACHINE_ALIASES; got $target_cc_machine" >&2; exit 3 ;; esac
target_tool_dir=$(dirname "$target_cc_path")
target_wrapper_root=$(dirname "$target_tool_dir")
target_cc_basename=${target_cc_path##*/}
case "$target_cc_basename" in *-gcc) target_tool_prefix=${target_cc_basename%-gcc} ;; *) target_tool_prefix= ;; esac
if [ -z "$target_tool_prefix" ]; then printf '%s\n' "target gcc name does not end in -gcc: $target_cc_basename" >&2; exit 3; fi
target_cxx_program=$target_tool_prefix-g++
target_ar_program=$target_tool_prefix-ar
target_ranlib_program=$target_tool_prefix-ranlib
target_objcopy_program=$target_tool_prefix-objcopy
if [ -n "$target_toolchain_root" ]; then
  for target_required_tool in "$target_tool_dir/$target_cxx_program" "$target_tool_dir/$target_ar_program" "$target_tool_dir/$target_ranlib_program" "$target_tool_dir/$target_objcopy_program"; do
    if [ ! -x "$target_required_tool" ]; then printf '%s\n' "source-root musl target toolchain root is incomplete: $target_required_tool" >&2; exit 3; fi
  done
  for target_required_file in "$target_toolchain_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libc.so" "$target_toolchain_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libgcc_s.so.1"; do
    if [ ! -f "$target_required_file" ]; then printf '%s\n' "source-root musl target toolchain root is incomplete: $target_required_file" >&2; exit 3; fi
  done
fi
target_nix_support="$target_wrapper_root/nix-support"
target_orig_libc_file="$target_nix_support/orig-libc"
target_orig_cc_file="$target_nix_support/orig-cc"
if [ -f "$target_orig_libc_file" ] && [ -f "$target_orig_cc_file" ]; then
  target_libc_root=$(cat "$target_orig_libc_file")
  target_orig_cc_root=$(cat "$target_orig_cc_file")
  target_gcc_crt_machine=$RUSTC_TARGET
else
  target_libc_root="$target_wrapper_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT"
  target_orig_cc_root="$target_wrapper_root"
  target_gcc_crt_machine=$target_cc_machine
fi
target_musl_crt_dir="$target_libc_root/lib"
target_gcc_version=$("$target_cc_path" -dumpfullversion 2>/dev/null || "$target_cc_path" -dumpversion 2>/dev/null || true)
target_gcc_crt_dir="$target_orig_cc_root/lib/gcc/$target_gcc_crt_machine/$target_gcc_version"
if [ ! -f "$target_gcc_crt_dir/crtbeginS.o" ]; then for candidate_dir in "$target_orig_cc_root"/lib/gcc/"$target_gcc_crt_machine"/*; do if [ -f "$candidate_dir/crtbeginS.o" ]; then target_gcc_crt_dir="$candidate_dir"; break; fi; done; fi
if [ ! -f "$target_musl_crt_dir/crt1.o" ] || [ ! -f "$target_musl_crt_dir/Scrt1.o" ] || [ ! -f "$target_musl_crt_dir/rcrt1.o" ] || [ ! -f "$target_musl_crt_dir/crti.o" ] || [ ! -f "$target_musl_crt_dir/crtn.o" ] || [ ! -f "$target_gcc_crt_dir/crtbeginS.o" ] || [ ! -f "$target_gcc_crt_dir/libgcc.a" ]; then printf '%s\n' 'target gcc toolchain does not expose musl/gcc CRT and unwinder objects' >&2; exit 3; fi
export NIX_CC_WRAPPER_TARGET_HOST_x86_64_unknown_linux_musl=1
export COPY_PROGRAM
target_alias_dir="$BUILD_DIR/target-linker-bin"
target_runtime_dir="$BUILD_DIR/target-linker-runtime"
mkdir -p "$target_alias_dir" "$target_runtime_dir"
for crt_name in crt1.o Scrt1.o rcrt1.o crti.o crtn.o; do rm -f "$target_runtime_dir/$crt_name"; $COPY_PROGRAM "$target_musl_crt_dir/$crt_name" "$target_runtime_dir/$crt_name"; done
for crt_name in crtbeginS.o crtendS.o; do rm -f "$target_runtime_dir/$crt_name"; $COPY_PROGRAM "$target_gcc_crt_dir/$crt_name" "$target_runtime_dir/$crt_name"; done
if [ ! -x "$target_tool_dir/$target_objcopy_program" ]; then printf '%s\n' 'target gcc toolchain does not expose objcopy for CRT normalization' >&2; exit 3; fi
target_crtbegin_no_frame_init="$target_runtime_dir/crtbeginS.o.no-frame-init"
rm -f "$target_crtbegin_no_frame_init"
"$target_tool_dir/$target_objcopy_program" --remove-section .init_array --remove-section .rela.init_array --remove-section .fini_array --remove-section .rela.fini_array "$target_runtime_dir/crtbeginS.o" "$target_crtbegin_no_frame_init"
rm -f "$target_runtime_dir/crtbeginS.o"
$COPY_PROGRAM "$target_crtbegin_no_frame_init" "$target_runtime_dir/crtbeginS.o"
rm -f "$target_crtbegin_no_frame_init"
target_unwind_archive="$target_gcc_crt_dir/libgcc_eh.a"
if [ ! -f "$target_unwind_archive" ]; then target_unwind_archive="$target_gcc_crt_dir/libgcc.a"; fi
rm -f "$target_runtime_dir/libunwind.a"; $COPY_PROGRAM "$target_unwind_archive" "$target_runtime_dir/libunwind.a"
rm -f "$target_runtime_dir/libgcc.a"; $COPY_PROGRAM "$target_gcc_crt_dir/libgcc.a" "$target_runtime_dir/libgcc.a"
rm -f "$target_runtime_dir/libgcc_s.a" "$target_runtime_dir/libgcc_s.so" "$target_runtime_dir/libgcc_s.so.1"
$COPY_PROGRAM "$target_gcc_crt_dir/libgcc.a" "$target_runtime_dir/libgcc_s.a"
for target_libgcc_shared_name in libgcc_s.so libgcc_s.so.1; do
  for target_libgcc_shared_dir in "$target_musl_crt_dir" "$target_orig_cc_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib" "$target_gcc_crt_dir"; do
    if [ -f "$target_libgcc_shared_dir/$target_libgcc_shared_name" ]; then $COPY_PROGRAM "$target_libgcc_shared_dir/$target_libgcc_shared_name" "$target_runtime_dir/$target_libgcc_shared_name"; break; fi
  done
done
target_lfs_compat_source="$target_runtime_dir/musl-lfs-compat.c"
target_lfs_compat_object="$target_runtime_dir/musl-lfs-compat.o"
cat > "$target_lfs_compat_source" <<'MANTLE_MUSL_LFS_COMPAT_C'
#define _LARGEFILE64_SOURCE 1
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <sys/mman.h>
#include <sys/sendfile.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#ifdef fstat64
#undef fstat64
#endif
#ifdef fstatat64
#undef fstatat64
#endif
#ifdef ftruncate64
#undef ftruncate64
#endif
#ifdef sendfile64
#undef sendfile64
#endif
#ifdef lseek64
#undef lseek64
#endif
#ifdef lstat64
#undef lstat64
#endif
#ifdef mmap64
#undef mmap64
#endif
#ifdef open64
#undef open64
#endif
#ifdef openat64
#undef openat64
#endif
#ifdef pread64
#undef pread64
#endif
#ifdef pwrite64
#undef pwrite64
#endif
#ifdef readdir64
#undef readdir64
#endif
#ifdef stat64
#undef stat64
#endif

#define MANTLE_PTHREAD_TLS_KEY_CAPACITY 4096u

typedef void (*mantle_pthread_tls_destructor)(void *);

static int mantle_pthread_tls_used[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static mantle_pthread_tls_destructor mantle_pthread_tls_destructors[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static __thread void *mantle_pthread_tls_values[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static unsigned int mantle_pthread_tls_next_key;

static int mantle_pthread_tls_key_is_valid(pthread_key_t key) {
    unsigned int index = (unsigned int)key;
    if (index >= MANTLE_PTHREAD_TLS_KEY_CAPACITY) {
        return 0;
    }
    if (__sync_add_and_fetch(&mantle_pthread_tls_used[index], 0) == 0) {
        return 0;
    }
    return 1;
}

int __wrap_pthread_key_create(pthread_key_t *key, void (*destructor)(void *)) {
    if (key == NULL) {
        return EINVAL;
    }
    for (unsigned int offset = 0; offset < MANTLE_PTHREAD_TLS_KEY_CAPACITY; offset++) {
        unsigned int candidate = (mantle_pthread_tls_next_key + offset) % MANTLE_PTHREAD_TLS_KEY_CAPACITY;
        if (__sync_bool_compare_and_swap(&mantle_pthread_tls_used[candidate], 0, 1) != 0) {
            mantle_pthread_tls_destructors[candidate] = destructor;
            *key = (pthread_key_t)candidate;
            mantle_pthread_tls_next_key = (candidate + 1) % MANTLE_PTHREAD_TLS_KEY_CAPACITY;
            return 0;
        }
    }
    return EAGAIN;
}

int __wrap_pthread_key_delete(pthread_key_t key) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return EINVAL;
    }
    unsigned int index = (unsigned int)key;
    mantle_pthread_tls_values[index] = NULL;
    mantle_pthread_tls_destructors[index] = NULL;
    __sync_lock_release(&mantle_pthread_tls_used[index]);
    return 0;
}

void *__wrap_pthread_getspecific(pthread_key_t key) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return NULL;
    }
    return mantle_pthread_tls_values[(unsigned int)key];
}

int __wrap_pthread_setspecific(pthread_key_t key, const void *value) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return EINVAL;
    }
    mantle_pthread_tls_values[(unsigned int)key] = (void *)value;
    return 0;
}

int backtrace(void **buffer, int size) {
    (void)buffer;
    (void)size;
    return 0;
}

void backtrace_symbols_fd(void *const *buffer, int size, int fd) {
    (void)buffer;
    (void)size;
    (void)fd;
}

static int mantle_open_flags_need_mode(int flags) {
    if ((flags & O_CREAT) != 0) {
        return 1;
    }
#ifdef O_TMPFILE
    if ((flags & O_TMPFILE) == O_TMPFILE) {
        return 1;
    }
#endif
    return 0;
}

int fstat64(int fd, struct stat *buf) {
    return fstat(fd, buf);
}

int fstatat64(int dirfd, const char *pathname, struct stat *buf, int flags) {
    return fstatat(dirfd, pathname, buf, flags);
}

int ftruncate64(int fd, off_t length) {
    return ftruncate(fd, length);
}

ssize_t sendfile64(int out_fd, int in_fd, off_t *offset, size_t count) {
    return sendfile(out_fd, in_fd, offset, count);
}

off_t lseek64(int fd, off_t offset, int whence) {
    return lseek(fd, offset, whence);
}

int lstat64(const char *pathname, struct stat *buf) {
    return lstat(pathname, buf);
}

void *mmap64(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    return mmap(addr, length, prot, flags, fd, offset);
}

int open64(const char *pathname, int flags, ...) {
    mode_t mode = 0;
    if (mantle_open_flags_need_mode(flags) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
        return open(pathname, flags, mode);
    }
    return open(pathname, flags);
}

int openat64(int dirfd, const char *pathname, int flags, ...) {
    mode_t mode = 0;
    if (mantle_open_flags_need_mode(flags) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
        return openat(dirfd, pathname, flags, mode);
    }
    return openat(dirfd, pathname, flags);
}

ssize_t pread64(int fd, void *buf, size_t count, off_t offset) {
    return pread(fd, buf, count, offset);
}

ssize_t pwrite64(int fd, const void *buf, size_t count, off_t offset) {
    return pwrite(fd, buf, count, offset);
}

struct dirent *readdir64(DIR *dirp) {
    return readdir(dirp);
}

int stat64(const char *pathname, struct stat *buf) {
    return stat(pathname, buf);
}
MANTLE_MUSL_LFS_COMPAT_C
"$target_cc_path" "$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" -D_LARGEFILE64_SOURCE -fno-asynchronous-unwind-tables -fPIC -c "$target_lfs_compat_source" -o "$target_lfs_compat_object"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/sh"
printf '%s\n' "exec \"$SHELL_PROGRAM\" \"\$@\"" >> "$target_alias_dir/sh"
chmod +x "$target_alias_dir/sh"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/emcc"
printf '%s\n' 'exit 127' >> "$target_alias_dir/emcc"
chmod +x "$target_alias_dir/emcc"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/pkg-config"
chmod +x "$target_alias_dir/pkg-config"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/pkgconf"
chmod +x "$target_alias_dir/pkgconf"
printf '%s\n' "#!$SHELL_PROGRAM" 'exit 127' > "$target_alias_dir/git"
chmod +x "$target_alias_dir/git"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/perl"
printf '%s\n' "exec \"$PERL_PROGRAM\" \"\$@\"" >> "$target_alias_dir/perl"
chmod +x "$target_alias_dir/perl"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/make"
printf '%s\n' "exec \"$MAKE_PROGRAM\" \"\$@\"" >> "$target_alias_dir/make"
chmod +x "$target_alias_dir/make"
printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/cc"
printf '%s\n' 'set -eu' >> "$target_alias_dir/cc"
printf '%s\n' "target_cc_path=\"\${MANTLE_TARGET_CC_PATH:-$target_cc_path}\"" >> "$target_alias_dir/cc"
printf '%s\n' "target_runtime_dir=\"$target_runtime_dir\"" >> "$target_alias_dir/cc"
printf '%s\n' 'static_pie_normalized=false' >> "$target_alias_dir/cc"
printf '%s\n' 'output_path=' >> "$target_alias_dir/cc"
printf '%s\n' 'previous_arg=' >> "$target_alias_dir/cc"
printf '%s\n' 'rustc_main_response_file=false' >> "$target_alias_dir/cc"
printf '%s\n' 'shared_link=false' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  if [ "$previous_arg" = "-o" ]; then output_path="$arg"; previous_arg=; continue; fi' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    -static-pie) static_pie_normalized=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -shared|-dynamiclib) shared_link=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -o) previous_arg=-o ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    @*/output/rustc-build/rustc_main_cmd.txt|@output/rustc-build/rustc_main_cmd.txt) rustc_main_response_file=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'dynamic_rustc_link=false' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$rustc_main_response_file" = true ]; then dynamic_rustc_link=true; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'case "$output_path" in' >> "$target_alias_dir/cc"
printf '%s\n' '  */output/rustc|output/rustc|*/output/rustc-build/rustc_main|output/rustc-build/rustc_main) dynamic_rustc_link=true ;;' >> "$target_alias_dir/cc"
printf '%s\n' 'esac' >> "$target_alias_dir/cc"
printf '%s\n' 'mapped_args_set=false' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    rcrt1.o|*/rcrt1.o) mapped_arg="$target_runtime_dir/crt1.o" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    crt1.o|*/crt1.o|Scrt1.o|*/Scrt1.o|crti.o|*/crti.o|crtn.o|*/crtn.o|crtbeginS.o|*/crtbeginS.o|crtendS.o|*/crtendS.o) mapped_arg="$target_runtime_dir/${arg##*/}" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -static) if [ "$shared_link" = true ]; then continue; else mapped_arg="$arg"; fi ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -static-pie) if [ "$dynamic_rustc_link" = true ] || [ "$shared_link" = true ]; then continue; else mapped_arg="-static"; fi ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    *) mapped_arg="$arg" ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' '  if [ "$mapped_args_set" = false ]; then set -- "$mapped_arg"; mapped_args_set=true; else set -- "$@" "$mapped_arg"; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$mapped_args_set" = false ]; then set --; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'link_command=true' >> "$target_alias_dir/cc"
printf '%s\n' 'static_support_link=true' >> "$target_alias_dir/cc"
printf '%s\n' 'for arg in "$@"; do' >> "$target_alias_dir/cc"
printf '%s\n' '  case "$arg" in' >> "$target_alias_dir/cc"
printf '%s\n' '    -c|-S|-E) link_command=false ;;' >> "$target_alias_dir/cc"
printf '%s\n' '    -shared|-dynamiclib) static_support_link=false ;;' >> "$target_alias_dir/cc"
printf '%s\n' '  esac' >> "$target_alias_dir/cc"
printf '%s\n' 'done' >> "$target_alias_dir/cc"
printf '%s\n' 'if [ "$link_command" = true ] && [ "$dynamic_rustc_link" = true ]; then set -- "$@" "$target_runtime_dir/musl-lfs-compat.o" -no-pie -Wl,-Bdynamic "-Wl,-dynamic-linker,$target_runtime_dir/libc.so" -Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; elif [ "$link_command" = true ] && [ "$static_support_link" = true ]; then set -- "$@" "$target_runtime_dir/musl-lfs-compat.o" -static -Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; fi' >> "$target_alias_dir/cc"
printf '%s\n' 'exec "$target_cc_path" "$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG" -D_LARGEFILE64_SOURCE -fno-asynchronous-unwind-tables -B"$target_runtime_dir/" -L"$target_runtime_dir" "$@"' >> "$target_alias_dir/cc"
chmod +x "$target_alias_dir/cc"
if [ -x "$target_tool_dir/$target_cxx_program" ]; then printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/c++"; printf '%s\n' 'set -eu' >> "$target_alias_dir/c++"; printf '%s\n' "MANTLE_TARGET_CC_PATH=\"$target_tool_dir/$target_cxx_program\"" >> "$target_alias_dir/c++"; printf '%s\n' 'export MANTLE_TARGET_CC_PATH' >> "$target_alias_dir/c++"; printf '%s\n' "exec \"$target_alias_dir/cc\" \"\$@\"" >> "$target_alias_dir/c++"; chmod +x "$target_alias_dir/c++"; fi
PATH="$target_alias_dir:$PATH"
export PATH
for target_tool in ar ranlib; do case "$target_tool" in ar) target_program="$target_ar_program" ;; ranlib) target_program="$target_ranlib_program" ;; esac; if [ -x "$target_tool_dir/$target_program" ]; then printf '%s\n' "#!$SHELL_PROGRAM" > "$target_alias_dir/$target_tool"; printf '%s\n' "exec \"$target_tool_dir/$target_program\" \"\$@\"" >> "$target_alias_dir/$target_tool"; chmod +x "$target_alias_dir/$target_tool"; fi; done
export CC_x86_64_linux_musl="$target_alias_dir/cc"
export CC_x86_64_unknown_linux_musl="$target_alias_dir/cc"
if [ -x "$target_alias_dir/c++" ]; then export CXX_x86_64_unknown_linux_musl="$target_alias_dir/c++"; fi
if [ -x "$target_alias_dir/ar" ]; then export AR_x86_64_unknown_linux_musl="$target_alias_dir/ar"; fi
if [ -x "$target_alias_dir/ranlib" ]; then export RANLIB_x86_64_unknown_linux_musl="$target_alias_dir/ranlib"; fi
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="$target_alias_dir/cc"
printf '%s\n' "using target linker wrapper: $target_alias_dir/cc -> $target_cc_path ($target_cc_machine); runtime CRT/unwind dir: $target_runtime_dir"
else
  printf '%s\n' "using compiler-host linker for $RUSTC_TARGET; target sysroot remains $RUSTC_PROVIDER_TARGET_TRIPLE"
fi
if [ "$RUSTC_TARGET" = "x86_64-unknown-linux-musl" ]; then RUN_RUSTC_DYLIB_EXT=rlib; else RUN_RUSTC_DYLIB_EXT=; fi
target_outdir_suffix=-target-"$RUSTC_TARGET"
target_prefix_s=run_rustc/output$target_outdir_suffix/prefix-s
target_libdir="$target_prefix_s/lib/rustlib/$RUSTC_TARGET/lib"
target_libstd="$target_libdir/libstd.rlib"
target_minicargo_flags="--target $RUSTC_TARGET"
target_bin_dir="$target_prefix_s/bin"
target_std_env_arch=${RUSTC_TARGET%%-*}
target_sysroot_source="rustc-${RUSTC_VERSION}-src/library/sysroot"
mkdir -p "$target_bin_dir" "$target_libdir"
$COPY_PROGRAM output/rustc "$target_bin_dir/rustc"
$COPY_PROGRAM output/cargo "$target_bin_dir/cargo"
chmod +x "$target_bin_dir/rustc" "$target_bin_dir/cargo"
STD_ENV_ARCH="$target_std_env_arch" MRUSTC_PATH="$(pwd)/$target_bin_dir/rustc" "$(pwd)/bin/minicargo" --vendor-dir "rustc-${RUSTC_VERSION}-src/vendor" --script-overrides "script-overrides/stable-${RUSTC_VERSION}-linux/" --output-dir "$target_libdir" $target_minicargo_flags "$target_sysroot_source"
if [ ! -f "$target_libstd" ]; then printf '%s\n' 'target rustlib build did not produce libstd.rlib' >&2; exit 3; fi
prefix_target_libdir=run_rustc/output/prefix/lib/rustlib/"$RUSTC_TARGET"/lib
mkdir -p "$prefix_target_libdir"
$COPY_PROGRAM "$target_libdir"/* "$prefix_target_libdir/"
RUSTC_TARGET="$RUSTC_HOST_TRIPLE"
else
  printf '%s\n' 'compiler host and provider target triples are identical; using host rustlib for target role'
fi
printf '%s\n' 'binding run_rustc stage tools to protected absolute executables'
run_rustc_stage_tool_makefile=run_rustc/Makefile
run_rustc_stage_tool_tmp="$BUILD_DIR/run-rustc-stage-tools.mk"
run_rustc_stage_tool_tab=$(printf '\t')
run_rustc_stage_rustc_original='$(BINDIR_S)rustc '
run_rustc_stage_rustc_patched='$(abspath $(BINDIR_S)rustc) '
run_rustc_stage_cargo_original='$(BINDIR_S)cargo '
run_rustc_stage_cargo_patched='$(abspath $(BINDIR_S)cargo) '
run_rustc_stage_rustc_seen=false
run_rustc_stage_cargo_seen=false
: > "$run_rustc_stage_tool_tmp"
while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    "$run_rustc_stage_tool_tab"*)
      case "$line" in
        *"$run_rustc_stage_rustc_original"*)
          run_rustc_stage_prefix=${line%%"$run_rustc_stage_rustc_original"*}
          run_rustc_stage_suffix=${line#*"$run_rustc_stage_rustc_original"}
          case "$run_rustc_stage_suffix" in *"$run_rustc_stage_rustc_original"*) printf '%s\n' 'run_rustc rustc recipe contains repeated executable tokens' >&2; exit 3 ;; esac
          printf '%s%s%s\n' "$run_rustc_stage_prefix" "$run_rustc_stage_rustc_patched" "$run_rustc_stage_suffix" >> "$run_rustc_stage_tool_tmp"
          run_rustc_stage_rustc_seen=true
          ;;
        *"$run_rustc_stage_cargo_original"*)
          run_rustc_stage_prefix=${line%%"$run_rustc_stage_cargo_original"*}
          run_rustc_stage_suffix=${line#*"$run_rustc_stage_cargo_original"}
          case "$run_rustc_stage_suffix" in *"$run_rustc_stage_cargo_original"*) printf '%s\n' 'run_rustc cargo recipe contains repeated executable tokens' >&2; exit 3 ;; esac
          printf '%s%s%s\n' "$run_rustc_stage_prefix" "$run_rustc_stage_cargo_patched" "$run_rustc_stage_suffix" >> "$run_rustc_stage_tool_tmp"
          run_rustc_stage_cargo_seen=true
          ;;
        *) printf '%s\n' "$line" >> "$run_rustc_stage_tool_tmp" ;;
      esac
      ;;
    *) printf '%s\n' "$line" >> "$run_rustc_stage_tool_tmp" ;;
  esac
done < "$run_rustc_stage_tool_makefile"
if [ "$run_rustc_stage_rustc_seen" = false ]; then printf '%s\n' 'run_rustc Makefile lacks a protected stage rustc recipe' >&2; exit 3; fi
if [ "$run_rustc_stage_cargo_seen" = false ]; then printf '%s\n' 'run_rustc Makefile lacks a protected stage cargo recipe' >&2; exit 3; fi
$COPY_PROGRAM "$run_rustc_stage_tool_tmp" "$run_rustc_stage_tool_makefile"
rm -f "$run_rustc_stage_tool_tmp"
if [ ! -f bin/mrustc ]; then printf '%s\n' 'mrustc build did not produce bin/mrustc' >&2; exit 3; fi
if [ ! -f bin/minicargo ]; then printf '%s\n' 'mrustc build did not produce bin/minicargo' >&2; exit 3; fi
if [ ! -f output/rustc ]; then printf '%s\n' 'mrustc build did not produce output/rustc' >&2; exit 3; fi
if [ ! -f output/cargo ]; then printf '%s\n' 'mrustc build did not produce output/cargo' >&2; exit 3; fi
if [ ! -f run_rustc/output/prefix/bin/rustc ]; then printf '%s\n' 'mrustc run_rustc did not produce prefix rustc' >&2; exit 3; fi
if [ ! -f run_rustc/output/prefix/bin/cargo ]; then printf '%s\n' 'mrustc run_rustc did not produce prefix cargo' >&2; exit 3; fi
printf '%s\n' "Rust 1.90 first-stage products ready"
