let Observation = {
  schema | String,
  target_path | String,
  target_exists | Bool,
  target_registered | Bool,
  origin_main | String,
  promotion_archive | String,
  promotion_archive_is_ancestor | Bool,
  facts_blake3 | String,
  removal_attempted | Bool,
  recursive_delete_attempted | Bool,
  branch_delete_attempted | Bool,
  decision | String,
}
in
{
  schema = "mantle-worktree-retirement-observation-v1",
  target_path = "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-durable-publication",
  target_exists = false,
  target_registered = false,
  origin_main = "6e670046c31fcc47169e857795c67a2b6a75bb69",
  promotion_archive = "bd614a7fd6fa90a49e278cf5b683edc17fe518f6",
  promotion_archive_is_ancestor = true,
  facts_blake3 = "61c4f848042ab53abe774239403faab76b49f020601863cb91d721bf43ac7dd8",
  removal_attempted = false,
  recursive_delete_attempted = false,
  branch_delete_attempted = false,
  decision = "obsolete-no-op",
} | Observation
