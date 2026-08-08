## snix-store NAR baseline

```text
$ env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-store --lib nar
copying path '/nix/store/9ypl7656d54kj4c8s2q7lzzfj9z976ir-source' from 'http://100.100.103.95:5000'...
copying path '/nix/store/q7in6vj1l6qnd10dn2r9xqynlxhx8s8c-source' from 'http://100.100.103.95:5000'...
copying path '/nix/store/hhccdmpa2bhqbagn2zgmgrwjbcdinn55-source' from 'http://100.100.103.95:5000'...
unpacking 'github:bytecodealliance/wasi-virt/19b174a3244f81ed9b91e067b6901f71665316a8?narHash=sha256-dj9FN941ejZXfbHsC2rxs7/2LSUeKQEf545aZ0QkEdI%3D' into the Git cache...
unpacking 'github:OnixResearch/octet/86ee46b3b9257b145d2dbeb6ce9d9897607db99c?narHash=sha256-%2BflXjhF9qVrXzPngauVj40t/nEF6927/qzgjRzfvzRw%3D' into the Git cache...
unpacking 'github:OnixResearch/valence/1f4f6fe903a34c22d2cda53cd534ce58bbf84147?narHash=sha256-uZi%2Bt9vXECcGahD9l8Hsfq3ssNVLqhU5%2BD%2B9ZGG67uc%3D' into the Git cache...
remote: Enumerating objects: 174, done.
remote: Counting objects:   0% (1/174)        remote: Counting objects:   1% (2/174)        remote: Counting objects:   2% (4/174)        remote: Counting objects:   3% (6/174)        remote: Counting objects:   4% (7/174)        remote: Counting objects:   5% (9/174)        remote: Counting objects:   6% (11/174)        remote: Counting objects:   7% (13/174)        remote: Counting objects:   8% (14/174)        remote: Counting objects:   9% (16/174)        remote: Counting objects:  10% (18/174)        remote: Counting objects:  11% (20/174)        remote: Counting objects:  12% (21/174)        remote: Counting objects:  13% (23/174)        remote: Counting objects:  14% (25/174)        remote: Counting objects:  15% (27/174)        remote: Counting objects:  16% (28/174)        remote: Counting objects:  17% (30/174)        remote: Counting objects:  18% (32/174)        remote: Counting objects:  19% (34/174)        remote: Counting objects:  20% (35/174)        remote: Counting objects:  21% (37/174)        remote: Counting objects:  22% (39/174)        remote: Counting objects:  23% (41/174)        remote: Counting objects:  24% (42/174)        remote: Counting objects:  25% (44/174)        remote: Counting objects:  26% (46/174)        remote: Counting objects:  27% (47/174)        remote: Counting objects:  28% (49/174)        remote: Counting objects:  29% (51/174)        remote: Counting objects:  30% (53/174)        remote: Counting objects:  31% (54/174)        remote: Counting objects:  32% (56/174)        remote: Counting objects:  33% (58/174)        remote: Counting objects:  34% (60/174)        remote: Counting objects:  35% (61/174)        remote: Counting objects:  36% (63/174)        remote: Counting objects:  37% (65/174)        remote: Counting objects:  38% (67/174)        remote: Counting objects:  39% (68/174)        remote: Counting objects:  40% (70/174)        remote: Counting objects:  41% (72/174)        remote: Counting objects:  42% (74/174)        remote: Counting objects:  43% (75/174)        remote: Counting objects:  44% (77/174)        remote: Counting objects:  45% (79/174)        remote: Counting objects:  46% (81/174)        remote: Counting objects:  47% (82/174)        remote: Counting objects:  48% (84/174)        remote: Counting objects:  49% (86/174)        remote: Counting objects:  50% (87/174)        remote: Counting objects:  51% (89/174)        remote: Counting objects:  52% (91/174)        remote: Counting objects:  53% (93/174)        remote: Counting objects:  54% (94/174)        remote: Counting objects:  55% (96/174)        remote: Counting objects:  56% (98/174)        remote: Counting objects:  57% (100/174)        remote: Counting objects:  58% (101/174)        remote: Counting objects:  59% (103/174)        remote: Counting objects:  60% (105/174)        remote: Counting objects:  61% (107/174)        remote: Counting objects:  62% (108/174)        remote: Counting objects:  63% (110/174)        remote: Counting objects:  64% (112/174)        remote: Counting objects:  65% (114/174)        remote: Counting objects:  66% (115/174)        remote: Counting objects:  67% (117/174)        remote: Counting objects:  68% (119/174)        remote: Counting objects:  69% (121/174)        remote: Counting objects:  70% (122/174)        remote: Counting objects:  71% (124/174)        remote: Counting objects:  72% (126/174)        remote: Counting objects:  73% (128/174)        remote: Counting objects:  74% (129/174)        remote: Counting objects:  75% (131/174)        remote: Counting objects:  76% (133/174)        remote: Counting objects:  77% (134/174)        remote: Counting objects:  78% (136/174)        remote: Counting objects:  79% (138/174)        remote: Counting objects:  80% (140/174)        remote: Counting objects:  81% (141/174)        remote: Counting objects:  82% (143/174)        remote: Counting objects:  83% (145/174)        remote: Counting objects:  84% (147/174)        remote: Counting objects:  85% (148/174)        remote: Counting objects:  86% (150/174)        remote: Counting objects:  87% (152/174)        remote: Counting objects:  88% (154/174)        remote: Counting objects:  89% (155/174)        remote: Counting objects:  90% (157/174)        remote: Counting objects:  91% (159/174)        remote: Counting objects:  92% (161/174)        remote: Counting objects:  93% (162/174)        remote: Counting objects:  94% (164/174)        remote: Counting objects:  95% (166/174)        remote: Counting objects:  96% (168/174)        remote: Counting objects:  97% (169/174)        remote: Counting objects:  98% (171/174)        remote: Counting objects:  99% (173/174)        remote: Counting objects: 100% (174/174)        remote: Counting objects: 100% (174/174), done.
remote: Compressing objects:   0% (1/139)        remote: Compressing objects:   1% (2/139)        remote: Compressing objects:   2% (3/139)        remote: Compressing objects:   3% (5/139)        remote: Compressing objects:   4% (6/139)        remote: Compressing objects:   5% (7/139)        remote: Compressing objects:   6% (9/139)        remote: Compressing objects:   7% (10/139)        remote: Compressing objects:   8% (12/139)        remote: Compressing objects:   9% (13/139)        remote: Compressing objects:  10% (14/139)        remote: Compressing objects:  11% (16/139)        remote: Compressing objects:  12% (17/139)        remote: Compressing objects:  13% (19/139)        remote: Compressing objects:  14% (20/139)        remote: Compressing objects:  15% (21/139)        remote: Compressing objects:  16% (23/139)        remote: Compressing objects:  17% (24/139)        remote: Compressing objects:  18% (26/139)        remote: Compressing objects:  19% (27/139)        remote: Compressing objects:  20% (28/139)        remote: Compressing objects:  21% (30/139)        remote: Compressing objects:  22% (31/139)        remote: Compressing objects:  23% (32/139)        remote: Compressing objects:  24% (34/139)        remote: Compressing objects:  25% (35/139)        remote: Compressing objects:  26% (37/139)        remote: Compressing objects:  27% (38/139)        remote: Compressing objects:  28% (39/139)        remote: Compressing objects:  29% (41/139)        remote: Compressing objects:  30% (42/139)        remote: Compressing objects:  31% (44/139)        remote: Compressing objects:  32% (45/139)        remote: Compressing objects:  33% (46/139)        remote: Compressing objects:  34% (48/139)        remote: Compressing objects:  35% (49/139)        remote: Compressing objects:  36% (51/139)        remote: Compressing objects:  37% (52/139)        remote: Compressing objects:  38% (53/139)        remote: Compressing objects:  39% (55/139)        remote: Compressing objects:  40% (56/139)        remote: Compressing objects:  41% (57/139)        remote: Compressing objects:  42% (59/139)        remote: Compressing objects:  43% (60/139)        remote: Compressing objects:  44% (62/139)        remote: Compressing objects:  45% (63/139)        remote: Compressing objects:  46% (64/139)        remote: Compressing objects:  47% (66/139)        remote: Compressing objects:  48% (67/139)        remote: Compressing objects:  49% (69/139)        remote: Compressing objects:  50% (70/139)        remote: Compressing objects:  51% (71/139)        remote: Compressing objects:  52% (73/139)        remote: Compressing objects:  53% (74/139)        remote: Compressing objects:  54% (76/139)        remote: Compressing objects:  55% (77/139)        remote: Compressing objects:  56% (78/139)        remote: Compressing objects:  57% (80/139)        remote: Compressing objects:  58% (81/139)        remote: Compressing objects:  59% (83/139)        remote: Compressing objects:  60% (84/139)        remote: Compressing objects:  61% (85/139)        remote: Compressing objects:  62% (87/139)        remote: Compressing objects:  63% (88/139)        remote: Compressing objects:  64% (89/139)        remote: Compressing objects:  65% (91/139)        remote: Compressing objects:  66% (92/139)        remote: Compressing objects:  67% (94/139)        remote: Compressing objects:  68% (95/139)        remote: Compressing objects:  69% (96/139)        remote: Compressing objects:  70% (98/139)        remote: Compressing objects:  71% (99/139)        remote: Compressing objects:  72% (101/139)        remote: Compressing objects:  73% (102/139)        remote: Compressing objects:  74% (103/139)        remote: Compressing objects:  75% (105/139)        remote: Compressing objects:  76% (106/139)        remote: Compressing objects:  77% (108/139)        remote: Compressing objects:  78% (109/139)        remote: Compressing objects:  79% (110/139)        remote: Compressing objects:  80% (112/139)        remote: Compressing objects:  81% (113/139)        remote: Compressing objects:  82% (114/139)        remote: Compressing objects:  83% (116/139)        remote: Compressing objects:  84% (117/139)        remote: Compressing objects:  85% (119/139)        remote: Compressing objects:  86% (120/139)        remote: Compressing objects:  87% (121/139)        remote: Compressing objects:  88% (123/139)        remote: Compressing objects:  89% (124/139)        remote: Compressing objects:  90% (126/139)        remote: Compressing objects:  91% (127/139)        remote: Compressing objects:  92% (128/139)        remote: Compressing objects:  93% (130/139)        remote: Compressing objects:  94% (131/139)        remote: Compressing objects:  95% (133/139)        remote: Compressing objects:  96% (134/139)        remote: Compressing objects:  97% (135/139)        remote: Compressing objects:  98% (137/139)        remote: Compressing objects:  99% (138/139)        remote: Compressing objects: 100% (139/139)        remote: Compressing objects: 100% (139/139), done.
Receiving objects:   0% (1/174)Receiving objects:   1% (2/174)Receiving objects:   2% (4/174)Receiving objects:   3% (6/174)Receiving objects:   4% (7/174)Receiving objects:   5% (9/174)Receiving objects:   6% (11/174)Receiving objects:   7% (13/174)Receiving objects:   8% (14/174)Receiving objects:   9% (16/174)Receiving objects:  10% (18/174)Receiving objects:  11% (20/174)Receiving objects:  12% (21/174)Receiving objects:  13% (23/174)Receiving objects:  14% (25/174)Receiving objects:  15% (27/174)Receiving objects:  16% (28/174)Receiving objects:  17% (30/174)Receiving objects:  18% (32/174)Receiving objects:  19% (34/174)Receiving objects:  20% (35/174)Receiving objects:  21% (37/174)Receiving objects:  22% (39/174)Receiving objects:  23% (41/174)Receiving objects:  24% (42/174)Receiving objects:  25% (44/174)Receiving objects:  26% (46/174)Receiving objects:  27% (47/174)Receiving objects:  28% (49/174)Receiving objects:  29% (51/174)Receiving objects:  30% (53/174)Receiving objects:  31% (54/174)Receiving objects:  32% (56/174)Receiving objects:  33% (58/174)Receiving objects:  34% (60/174)Receiving objects:  35% (61/174)Receiving objects:  36% (63/174)Receiving objects:  37% (65/174)Receiving objects:  38% (67/174)Receiving objects:  39% (68/174)Receiving objects:  40% (70/174)Receiving objects:  41% (72/174)Receiving objects:  42% (74/174)Receiving objects:  43% (75/174)Receiving objects:  44% (77/174)Receiving objects:  45% (79/174)Receiving objects:  46% (81/174)Receiving objects:  47% (82/174)Receiving objects:  48% (84/174)Receiving objects:  49% (86/174)Receiving objects:  50% (87/174)Receiving objects:  51% (89/174)Receiving objects:  52% (91/174)Receiving objects:  53% (93/174)Receiving objects:  54% (94/174)Receiving objects:  55% (96/174)Receiving objects:  56% (98/174)Receiving objects:  57% (100/174)Receiving objects:  58% (101/174)Receiving objects:  59% (103/174)Receiving objects:  60% (105/174)Receiving objects:  61% (107/174)Receiving objects:  62% (108/174)Receiving objects:  63% (110/174)Receiving objects:  64% (112/174)Receiving objects:  65% (114/174)Receiving objects:  66% (115/174)Receiving objects:  67% (117/174)Receiving objects:  68% (119/174)Receiving objects:  69% (121/174)Receiving objects:  70% (122/174)Receiving objects:  71% (124/174)Receiving objects:  72% (126/174)Receiving objects:  73% (128/174)Receiving objects:  74% (129/174)Receiving objects:  75% (131/174)Receiving objects:  76% (133/174)Receiving objects:  77% (134/174)Receiving objects:  78% (136/174)Receiving objects:  79% (138/174)Receiving objects:  80% (140/174)Receiving objects:  81% (141/174)Receiving objects:  82% (143/174)Receiving objects:  83% (145/174)Receiving objects:  84% (147/174)Receiving objects:  85% (148/174)Receiving objects:  86% (150/174)Receiving objects:  87% (152/174)Receiving objects:  88% (154/174)Receiving objects:  89% (155/174)Receiving objects:  90% (157/174)Receiving objects:  91% (159/174)remote: Total 174 (delta 14), reused 144 (delta 9), pack-reused 0 (from 0)
Receiving objects:  92% (161/174)Receiving objects:  93% (162/174)Receiving objects:  94% (164/174)Receiving objects:  95% (166/174)Receiving objects:  96% (168/174)Receiving objects:  97% (169/174)Receiving objects:  98% (171/174)Receiving objects:  99% (173/174)Receiving objects: 100% (174/174)Receiving objects: 100% (174/174), 182.58 KiB | 7.94 MiB/s, done.
Resolving deltas:   0% (0/14)Resolving deltas:   7% (1/14)Resolving deltas:  14% (2/14)Resolving deltas:  21% (3/14)Resolving deltas:  28% (4/14)Resolving deltas:  35% (5/14)Resolving deltas:  42% (6/14)Resolving deltas:  57% (8/14)Resolving deltas:  64% (9/14)Resolving deltas:  71% (10/14)Resolving deltas:  78% (11/14)Resolving deltas:  92% (13/14)Resolving deltas: 100% (14/14)Resolving deltas: 100% (14/14), done.
From ssh://github.com/OnixResearch/onix-artifact
 * branch            c932138d880ddf4c2967f4c024b489b5c0022bf1 -> FETCH_HEAD
fatal: repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git/' not found
error:
       … while calling the 'derivationStrict' builtin
         at «nix-internal»/derivation-internal.nix:37:12:
           36|
           37|   strict = derivationStrict drvAttrs;
             |            ^
           38|

       … while evaluating derivation 'nix-shell'
         whose name attribute is located at «github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e?narHash=sha256-/m3yyS/EnXqoPGBJYVy4jTOsirdgsEZ3JdN2gGkBr14%3D»/pkgs/stdenv/generic/make-derivation.nix:541:11

       … while evaluating attribute 'buildInputs' of derivation 'nix-shell'
         at «github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e?narHash=sha256-/m3yyS/EnXqoPGBJYVy4jTOsirdgsEZ3JdN2gGkBr14%3D»/pkgs/stdenv/generic/make-derivation.nix:593:11:
          592|           depsHostHost = elemAt (elemAt dependencies 1) 0;
          593|           buildInputs = elemAt (elemAt dependencies 1) 1;
             |           ^
          594|           depsTargetTarget = elemAt (elemAt dependencies 2) 0;

       (stack trace truncated; use '--show-trace' to show the full, detailed trace)

       error: Failed to fetch git repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'

exit_status: 1
```

## crunch-store baseline

```text
$ env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib
fatal: repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git/' not found
error:
       … while calling the 'derivationStrict' builtin
         at «nix-internal»/derivation-internal.nix:37:12:
           36|
           37|   strict = derivationStrict drvAttrs;
             |            ^
           38|

       … while evaluating derivation 'nix-shell'
         whose name attribute is located at «github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e?narHash=sha256-/m3yyS/EnXqoPGBJYVy4jTOsirdgsEZ3JdN2gGkBr14%3D»/pkgs/stdenv/generic/make-derivation.nix:541:11

       … while evaluating attribute 'buildInputs' of derivation 'nix-shell'
         at «github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e?narHash=sha256-/m3yyS/EnXqoPGBJYVy4jTOsirdgsEZ3JdN2gGkBr14%3D»/pkgs/stdenv/generic/make-derivation.nix:593:11:
          592|           depsHostHost = elemAt (elemAt dependencies 1) 0;
          593|           buildInputs = elemAt (elemAt dependencies 1) 1;
             |           ^
          594|           depsTargetTarget = elemAt (elemAt dependencies 2) 0;

       (stack trace truncated; use '--show-trace' to show the full, detailed trace)

       error: Failed to fetch git repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'

exit_status: 1
```

## project refresh CLI baseline

```text
$ env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --test project_refresh_cli
fatal: repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git/' not found
error:
       … while calling the 'derivationStrict' builtin
         at «nix-internal»/derivation-internal.nix:37:12:
           36|
           37|   strict = derivationStrict drvAttrs;
             |            ^
           38|

       … while evaluating derivation 'nix-shell'
         whose name attribute is located at «github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e?narHash=sha256-/m3yyS/EnXqoPGBJYVy4jTOsirdgsEZ3JdN2gGkBr14%3D»/pkgs/stdenv/generic/make-derivation.nix:541:11

       … while evaluating attribute 'buildInputs' of derivation 'nix-shell'
         at «github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e?narHash=sha256-/m3yyS/EnXqoPGBJYVy4jTOsirdgsEZ3JdN2gGkBr14%3D»/pkgs/stdenv/generic/make-derivation.nix:593:11:
          592|           depsHostHost = elemAt (elemAt dependencies 1) 0;
          593|           buildInputs = elemAt (elemAt dependencies 1) 1;
             |           ^
          594|           depsTargetTarget = elemAt (elemAt dependencies 2) 0;

       (stack trace truncated; use '--show-trace' to show the full, detailed trace)

       error: Failed to fetch git repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'

exit_status: 1
```

## Authorized local source retry

The Nix input URL was unavailable. This retry maps only that exact URL to the local sibling repository.

The sibling contains commit `951c27f59003cea9bfdb40ed4d89653d50fada1f`. No lock file changed.

### snix-store NAR baseline retry

```text
$ env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-store --lib nar
remote: Enumerating objects: 60, done.
remote: Counting objects:   1% (1/60)        remote: Counting objects:   3% (2/60)        remote: Counting objects:   5% (3/60)        remote: Counting objects:   6% (4/60)        remote: Counting objects:   8% (5/60)        remote: Counting objects:  10% (6/60)        remote: Counting objects:  11% (7/60)        remote: Counting objects:  13% (8/60)        remote: Counting objects:  15% (9/60)        remote: Counting objects:  16% (10/60)        remote: Counting objects:  18% (11/60)        remote: Counting objects:  20% (12/60)        remote: Counting objects:  21% (13/60)        remote: Counting objects:  23% (14/60)        remote: Counting objects:  25% (15/60)        remote: Counting objects:  26% (16/60)        remote: Counting objects:  28% (17/60)        remote: Counting objects:  30% (18/60)        remote: Counting objects:  31% (19/60)        remote: Counting objects:  33% (20/60)        remote: Counting objects:  35% (21/60)        remote: Counting objects:  36% (22/60)        remote: Counting objects:  38% (23/60)        remote: Counting objects:  40% (24/60)        remote: Counting objects:  41% (25/60)        remote: Counting objects:  43% (26/60)        remote: Counting objects:  45% (27/60)        remote: Counting objects:  46% (28/60)        remote: Counting objects:  48% (29/60)        remote: Counting objects:  50% (30/60)        remote: Counting objects:  51% (31/60)        remote: Counting objects:  53% (32/60)        remote: Counting objects:  55% (33/60)        remote: Counting objects:  56% (34/60)        remote: Counting objects:  58% (35/60)        remote: Counting objects:  60% (36/60)        remote: Counting objects:  61% (37/60)        remote: Counting objects:  63% (38/60)        remote: Counting objects:  65% (39/60)        remote: Counting objects:  66% (40/60)        remote: Counting objects:  68% (41/60)        remote: Counting objects:  70% (42/60)        remote: Counting objects:  71% (43/60)        remote: Counting objects:  73% (44/60)        remote: Counting objects:  75% (45/60)        remote: Counting objects:  76% (46/60)        remote: Counting objects:  78% (47/60)        remote: Counting objects:  80% (48/60)        remote: Counting objects:  81% (49/60)        remote: Counting objects:  83% (50/60)        remote: Counting objects:  85% (51/60)        remote: Counting objects:  86% (52/60)        remote: Counting objects:  88% (53/60)        remote: Counting objects:  90% (54/60)        remote: Counting objects:  91% (55/60)        remote: Counting objects:  93% (56/60)        remote: Counting objects:  95% (57/60)        remote: Counting objects:  96% (58/60)        remote: Counting objects:  98% (59/60)        remote: Counting objects: 100% (60/60)        remote: Counting objects: 100% (60/60), done.
remote: Compressing objects:   1% (1/54)        remote: Compressing objects:   3% (2/54)        remote: Compressing objects:   5% (3/54)        remote: Compressing objects:   7% (4/54)        remote: Compressing objects:   9% (5/54)        remote: Compressing objects:  11% (6/54)        remote: Compressing objects:  12% (7/54)        remote: Compressing objects:  14% (8/54)        remote: Compressing objects:  16% (9/54)        remote: Compressing objects:  18% (10/54)        remote: Compressing objects:  20% (11/54)        remote: Compressing objects:  22% (12/54)        remote: Compressing objects:  24% (13/54)        remote: Compressing objects:  25% (14/54)        remote: Compressing objects:  27% (15/54)        remote: Compressing objects:  29% (16/54)        remote: Compressing objects:  31% (17/54)        remote: Compressing objects:  33% (18/54)        remote: Compressing objects:  35% (19/54)        remote: Compressing objects:  37% (20/54)        remote: Compressing objects:  38% (21/54)        remote: Compressing objects:  40% (22/54)        remote: Compressing objects:  42% (23/54)        remote: Compressing objects:  44% (24/54)        remote: Compressing objects:  46% (25/54)        remote: Compressing objects:  48% (26/54)        remote: Compressing objects:  50% (27/54)        remote: Compressing objects:  51% (28/54)        remote: Compressing objects:  53% (29/54)        remote: Compressing objects:  55% (30/54)        remote: Compressing objects:  57% (31/54)        remote: Compressing objects:  59% (32/54)        remote: Compressing objects:  61% (33/54)        remote: Compressing objects:  62% (34/54)        remote: Compressing objects:  64% (35/54)        remote: Compressing objects:  66% (36/54)        remote: Compressing objects:  68% (37/54)        remote: Compressing objects:  70% (38/54)        remote: Compressing objects:  72% (39/54)        remote: Compressing objects:  74% (40/54)        remote: Compressing objects:  75% (41/54)        remote: Compressing objects:  77% (42/54)        remote: Compressing objects:  79% (43/54)        remote: Compressing objects:  81% (44/54)        remote: Compressing objects:  83% (45/54)        remote: Compressing objects:  85% (46/54)        remote: Compressing objects:  87% (47/54)        remote: Compressing objects:  88% (48/54)        remote: Compressing objects:  90% (49/54)        remote: Compressing objects:  92% (50/54)        remote: Compressing objects:  94% (51/54)        remote: Compressing objects:  96% (52/54)        remote: Compressing objects:  98% (53/54)        remote: Compressing objects: 100% (54/54)        remote: Compressing objects: 100% (54/54), done.
remote: Total 60 (delta 3), reused 0 (delta 0), pack-reused 0 (from 0)
From file:///home/brittonr/git/OnixResearch/durable-file-publication
 * branch            951c27f59003cea9bfdb40ed4d89653d50fada1f -> FETCH_HEAD
remote: Enumerating objects: 44, done.
remote: Counting objects:   2% (1/44)        remote: Counting objects:   4% (2/44)        remote: Counting objects:   6% (3/44)        remote: Counting objects:   9% (4/44)        remote: Counting objects:  11% (5/44)        remote: Counting objects:  13% (6/44)        remote: Counting objects:  15% (7/44)        remote: Counting objects:  18% (8/44)        remote: Counting objects:  20% (9/44)        remote: Counting objects:  22% (10/44)        remote: Counting objects:  25% (11/44)        remote: Counting objects:  27% (12/44)        remote: Counting objects:  29% (13/44)        remote: Counting objects:  31% (14/44)        remote: Counting objects:  34% (15/44)        remote: Counting objects:  36% (16/44)        remote: Counting objects:  38% (17/44)        remote: Counting objects:  40% (18/44)        remote: Counting objects:  43% (19/44)        remote: Counting objects:  45% (20/44)        remote: Counting objects:  47% (21/44)        remote: Counting objects:  50% (22/44)        remote: Counting objects:  52% (23/44)        remote: Counting objects:  54% (24/44)        remote: Counting objects:  56% (25/44)        remote: Counting objects:  59% (26/44)        remote: Counting objects:  61% (27/44)        remote: Counting objects:  63% (28/44)        remote: Counting objects:  65% (29/44)        remote: Counting objects:  68% (30/44)        remote: Counting objects:  70% (31/44)        remote: Counting objects:  72% (32/44)        remote: Counting objects:  75% (33/44)        remote: Counting objects:  77% (34/44)        remote: Counting objects:  79% (35/44)        remote: Counting objects:  81% (36/44)        remote: Counting objects:  84% (37/44)        remote: Counting objects:  86% (38/44)        remote: Counting objects:  88% (39/44)        remote: Counting objects:  90% (40/44)        remote: Counting objects:  93% (41/44)        remote: Counting objects:  95% (42/44)        remote: Counting objects:  97% (43/44)        remote: Counting objects: 100% (44/44)        remote: Counting objects: 100% (44/44), done.
remote: Compressing objects:   4% (1/25)        remote: Compressing objects:   8% (2/25)        remote: Compressing objects:  12% (3/25)        remote: Compressing objects:  16% (4/25)        remote: Compressing objects:  20% (5/25)        remote: Compressing objects:  24% (6/25)        remote: Compressing objects:  28% (7/25)        remote: Compressing objects:  32% (8/25)        remote: Compressing objects:  36% (9/25)        remote: Compressing objects:  40% (10/25)        remote: Compressing objects:  44% (11/25)        remote: Compressing objects:  48% (12/25)        remote: Compressing objects:  52% (13/25)        remote: Compressing objects:  56% (14/25)        remote: Compressing objects:  60% (15/25)        remote: Compressing objects:  64% (16/25)        remote: Compressing objects:  68% (17/25)        remote: Compressing objects:  72% (18/25)        remote: Compressing objects:  76% (19/25)        remote: Compressing objects:  80% (20/25)        remote: Compressing objects:  84% (21/25)        remote: Compressing objects:  88% (22/25)        remote: Compressing objects:  92% (23/25)        remote: Compressing objects:  96% (24/25)        remote: Compressing objects: 100% (25/25)        remote: Compressing objects: 100% (25/25), done.
remote: Total 44 (delta 15), reused 41 (delta 12), pack-reused 0 (from 0)
From https://github.com/tvlfyi/wu-manber
 * [new branch]      master           -> master
 * [new ref]         refs/pull/1/head -> refs/pull/1/head
remote: Enumerating objects: 646, done.
remote: Counting objects:   0% (1/646)        remote: Counting objects:   1% (7/646)        remote: Counting objects:   2% (13/646)        remote: Counting objects:   3% (20/646)        remote: Counting objects:   4% (26/646)        remote: Counting objects:   5% (33/646)        remote: Counting objects:   6% (39/646)        remote: Counting objects:   7% (46/646)        remote: Counting objects:   8% (52/646)        remote: Counting objects:   9% (59/646)        remote: Counting objects:  10% (65/646)        remote: Counting objects:  11% (72/646)        remote: Counting objects:  12% (78/646)        remote: Counting objects:  13% (84/646)        remote: Counting objects:  14% (91/646)        remote: Counting objects:  15% (97/646)        remote: Counting objects:  16% (104/646)        remote: Counting objects:  17% (110/646)        remote: Counting objects:  18% (117/646)        remote: Counting objects:  19% (123/646)        remote: Counting objects:  20% (130/646)        remote: Counting objects:  21% (136/646)        remote: Counting objects:  22% (143/646)        remote: Counting objects:  23% (149/646)        remote: Counting objects:  24% (156/646)        remote: Counting objects:  25% (162/646)        remote: Counting objects:  26% (168/646)        remote: Counting objects:  27% (175/646)        remote: Counting objects:  28% (181/646)        remote: Counting objects:  29% (188/646)        remote: Counting objects:  30% (194/646)        remote: Counting objects:  31% (201/646)        remote: Counting objects:  32% (207/646)        remote: Counting objects:  33% (214/646)        remote: Counting objects:  34% (220/646)        remote: Counting objects:  35% (227/646)        remote: Counting objects:  36% (233/646)        remote: Counting objects:  37% (240/646)        remote: Counting objects:  38% (246/646)        remote: Counting objects:  39% (252/646)        remote: Counting objects:  40% (259/646)        remote: Counting objects:  41% (265/646)        remote: Counting objects:  42% (272/646)        remote: Counting objects:  43% (278/646)        remote: Counting objects:  44% (285/646)        remote: Counting objects:  45% (291/646)        remote: Counting objects:  46% (298/646)        remote: Counting objects:  47% (304/646)        remote: Counting objects:  48% (311/646)        remote: Counting objects:  49% (317/646)        remote: Counting objects:  50% (323/646)        remote: Counting objects:  51% (330/646)        remote: Counting objects:  52% (336/646)        remote: Counting objects:  53% (343/646)        remote: Counting objects:  54% (349/646)        remote: Counting objects:  55% (356/646)        remote: Counting objects:  56% (362/646)        remote: Counting objects:  57% (369/646)        remote: Counting objects:  58% (375/646)        remote: Counting objects:  59% (382/646)        remote: Counting objects:  60% (388/646)        remote: Counting objects:  61% (395/646)        remote: Counting objects:  62% (401/646)        remote: Counting objects:  63% (407/646)        remote: Counting objects:  64% (414/646)        remote: Counting objects:  65% (420/646)        remote: Counting objects:  66% (427/646)        remote: Counting objects:  67% (433/646)        remote: Counting objects:  68% (440/646)        remote: Counting objects:  69% (446/646)        remote: Counting objects:  70% (453/646)        remote: Counting objects:  71% (459/646)        remote: Counting objects:  72% (466/646)        remote: Counting objects:  73% (472/646)        remote: Counting objects:  74% (479/646)        remote: Counting objects:  75% (485/646)        remote: Counting objects:  76% (491/646)        remote: Counting objects:  77% (498/646)        remote: Counting objects:  78% (504/646)        remote: Counting objects:  79% (511/646)        remote: Counting objects:  80% (517/646)        remote: Counting objects:  81% (524/646)        remote: Counting objects:  82% (530/646)        remote: Counting objects:  83% (537/646)        remote: Counting objects:  84% (543/646)        remote: Counting objects:  85% (550/646)        remote: Counting objects:  86% (556/646)        remote: Counting objects:  87% (563/646)        remote: Counting objects:  88% (569/646)        remote: Counting objects:  89% (575/646)        remote: Counting objects:  90% (582/646)        remote: Counting objects:  91% (588/646)        remote: Counting objects:  92% (595/646)        remote: Counting objects:  93% (601/646)        remote: Counting objects:  94% (608/646)        remote: Counting objects:  95% (614/646)        remote: Counting objects:  96% (621/646)        remote: Counting objects:  97% (627/646)        remote: Counting objects:  98% (634/646)        remote: Counting objects:  99% (640/646)        remote: Counting objects: 100% (646/646)        remote: Counting objects: 100% (646/646), done.
remote: Compressing objects:   0% (1/328)        remote: Compressing objects:   1% (4/328)        remote: Compressing objects:   2% (7/328)        remote: Compressing objects:   3% (10/328)        remote: Compressing objects:   4% (14/328)        remote: Compressing objects:   5% (17/328)        remote: Compressing objects:   6% (20/328)        remote: Compressing objects:   7% (23/328)        remote: Compressing objects:   8% (27/328)        remote: Compressing objects:   9% (30/328)        remote: Compressing objects:  10% (33/328)        remote: Compressing objects:  11% (37/328)        remote: Compressing objects:  12% (40/328)        remote: Compressing objects:  13% (43/328)        remote: Compressing objects:  14% (46/328)        remote: Compressing objects:  15% (50/328)        remote: Compressing objects:  16% (53/328)        remote: Compressing objects:  17% (56/328)        remote: Compressing objects:  18% (60/328)        remote: Compressing objects:  19% (63/328)        remote: Compressing objects:  20% (66/328)        remote: Compressing objects:  21% (69/328)        remote: Compressing objects:  22% (73/328)        remote: Compressing objects:  23% (76/328)        remote: Compressing objects:  24% (79/328)        remote: Compressing objects:  25% (82/328)        remote: Compressing objects:  26% (86/328)        remote: Compressing objects:  27% (89/328)        remote: Compressing objects:  28% (92/328)        remote: Compressing objects:  29% (96/328)        remote: Compressing objects:  30% (99/328)        remote: Compressing objects:  31% (102/328)        remote: Compressing objects:  32% (105/328)        remote: Compressing objects:  33% (109/328)        remote: Compressing objects:  34% (112/328)        remote: Compressing objects:  35% (115/328)        remote: Compressing objects:  36% (119/328)        remote: Compressing objects:  37% (122/328)        remote: Compressing objects:  38% (125/328)        remote: Compressing objects:  39% (128/328)        remote: Compressing objects:  40% (132/328)        remote: Compressing objects:  41% (135/328)        remote: Compressing objects:  42% (138/328)        remote: Compressing objects:  43% (142/328)        remote: Compressing objects:  44% (145/328)        remote: Compressing objects:  45% (148/328)        remote: Compressing objects:  46% (151/328)        remote: Compressing objects:  47% (155/328)        remote: Compressing objects:  48% (158/328)        remote: Compressing objects:  49% (161/328)        remote: Compressing objects:  50% (164/328)        remote: Compressing objects:  51% (168/328)        remote: Compressing objects:  52% (171/328)        remote: Compressing objects:  53% (174/328)        remote: Compressing objects:  54% (178/328)        remote: Compressing objects:  55% (181/328)        remote: Compressing objects:  56% (184/328)        remote: Compressing objects:  57% (187/328)        remote: Compressing objects:  58% (191/328)        remote: Compressing objects:  59% (194/328)        remote: Compressing objects:  60% (197/328)        remote: Compressing objects:  61% (201/328)        remote: Compressing objects:  62% (204/328)        remote: Compressing objects:  63% (207/328)        remote: Compressing objects:  64% (210/328)        remote: Compressing objects:  65% (214/328)        remote: Compressing objects:  66% (217/328)        remote: Compressing objects:  67% (220/328)        remote: Compressing objects:  68% (224/328)        remote: Compressing objects:  69% (227/328)        remote: Compressing objects:  70% (230/328)        remote: Compressing objects:  71% (233/328)        remote: Compressing objects:  72% (237/328)        remote: Compressing objects:  73% (240/328)        remote: Compressing objects:  74% (243/328)        remote: Compressing objects:  75% (246/328)        remote: Compressing objects:  76% (250/328)        remote: Compressing objects:  77% (253/328)        remote: Compressing objects:  78% (256/328)        remote: Compressing objects:  79% (260/328)        remote: Compressing objects:  80% (263/328)        remote: Compressing objects:  81% (266/328)        remote: Compressing objects:  82% (269/328)        remote: Compressing objects:  83% (273/328)        remote: Compressing objects:  84% (276/328)        remote: Compressing objects:  85% (279/328)        remote: Compressing objects:  86% (283/328)        remote: Compressing objects:  87% (286/328)        remote: Compressing objects:  88% (289/328)        remote: Compressing objects:  89% (292/328)        remote: Compressing objects:  90% (296/328)        remote: Compressing objects:  91% (299/328)        remote: Compressing objects:  92% (302/328)        remote: Compressing objects:  93% (306/328)        remote: Compressing objects:  94% (309/328)        remote: Compressing objects:  95% (312/328)        remote: Compressing objects:  96% (315/328)        remote: Compressing objects:  97% (319/328)        remote: Compressing objects:  98% (322/328)        remote: Compressing objects:  99% (325/328)        remote: Compressing objects: 100% (328/328)        remote: Compressing objects: 100% (328/328), done.
Receiving objects:   0% (1/646)Receiving objects:   1% (7/646)Receiving objects:   2% (13/646)Receiving objects:   3% (20/646)Receiving objects:   4% (26/646)Receiving objects:   5% (33/646)Receiving objects:   6% (39/646)Receiving objects:   7% (46/646)Receiving objects:   8% (52/646)Receiving objects:   9% (59/646)Receiving objects:  10% (65/646)Receiving objects:  11% (72/646)Receiving objects:  12% (78/646)Receiving objects:  13% (84/646)Receiving objects:  14% (91/646)Receiving objects:  15% (97/646)Receiving objects:  16% (104/646)Receiving objects:  17% (110/646)Receiving objects:  18% (117/646)Receiving objects:  19% (123/646)Receiving objects:  20% (130/646)Receiving objects:  21% (136/646)Receiving objects:  22% (143/646)Receiving objects:  23% (149/646)Receiving objects:  24% (156/646)Receiving objects:  25% (162/646)Receiving objects:  26% (168/646)Receiving objects:  27% (175/646)Receiving objects:  28% (181/646)Receiving objects:  29% (188/646)Receiving objects:  30% (194/646)Receiving objects:  31% (201/646)Receiving objects:  32% (207/646)Receiving objects:  33% (214/646)Receiving objects:  34% (220/646)Receiving objects:  35% (227/646)Receiving objects:  36% (233/646)Receiving objects:  37% (240/646)Receiving objects:  38% (246/646)Receiving objects:  39% (252/646)Receiving objects:  40% (259/646)Receiving objects:  41% (265/646)Receiving objects:  42% (272/646)Receiving objects:  43% (278/646)Receiving objects:  44% (285/646)Receiving objects:  45% (291/646)Receiving objects:  46% (298/646)Receiving objects:  47% (304/646)Receiving objects:  48% (311/646)Receiving objects:  49% (317/646)Receiving objects:  50% (323/646)Receiving objects:  51% (330/646)Receiving objects:  52% (336/646)Receiving objects:  53% (343/646)Receiving objects:  54% (349/646)Receiving objects:  55% (356/646)Receiving objects:  56% (362/646)Receiving objects:  57% (369/646)Receiving objects:  58% (375/646)Receiving objects:  59% (382/646)Receiving objects:  60% (388/646)Receiving objects:  61% (395/646)Receiving objects:  62% (401/646)Receiving objects:  63% (407/646)Receiving objects:  64% (414/646)Receiving objects:  65% (420/646)Receiving objects:  66% (427/646)Receiving objects:  67% (433/646)Receiving objects:  68% (440/646)Receiving objects:  69% (446/646)Receiving objects:  70% (453/646)Receiving objects:  71% (459/646)Receiving objects:  72% (466/646)Receiving objects:  73% (472/646)Receiving objects:  74% (479/646)Receiving objects:  75% (485/646)Receiving objects:  76% (491/646)Receiving objects:  77% (498/646)Receiving objects:  78% (504/646)Receiving objects:  79% (511/646)Receiving objects:  80% (517/646)Receiving objects:  81% (524/646)Receiving objects:  82% (530/646)Receiving objects:  83% (537/646)Receiving objects:  84% (543/646)remote: Total 646 (delta 265), reused 625 (delta 244), pack-reused 0 (from 0)
Receiving objects:  85% (550/646)Receiving objects:  86% (556/646)Receiving objects:  87% (563/646)Receiving objects:  88% (569/646)Receiving objects:  89% (575/646)Receiving objects:  90% (582/646)Receiving objects:  91% (588/646)Receiving objects:  92% (595/646)Receiving objects:  93% (601/646)Receiving objects:  94% (608/646)Receiving objects:  95% (614/646)Receiving objects:  96% (621/646)Receiving objects:  97% (627/646)Receiving objects:  98% (634/646)Receiving objects:  99% (640/646)Receiving objects: 100% (646/646)Receiving objects: 100% (646/646), 211.96 KiB | 6.62 MiB/s, done.
Resolving deltas:   0% (0/265)Resolving deltas:   1% (3/265)Resolving deltas:   2% (6/265)Resolving deltas:   3% (8/265)Resolving deltas:   4% (11/265)Resolving deltas:   5% (14/265)Resolving deltas:   6% (16/265)Resolving deltas:   7% (19/265)Resolving deltas:   8% (22/265)Resolving deltas:   9% (24/265)Resolving deltas:  10% (27/265)Resolving deltas:  11% (30/265)Resolving deltas:  12% (34/265)Resolving deltas:  13% (35/265)Resolving deltas:  14% (38/265)Resolving deltas:  15% (41/265)Resolving deltas:  16% (43/265)Resolving deltas:  17% (46/265)Resolving deltas:  18% (49/265)Resolving deltas:  19% (51/265)Resolving deltas:  20% (53/265)Resolving deltas:  21% (56/265)Resolving deltas:  22% (59/265)Resolving deltas:  23% (62/265)Resolving deltas:  24% (64/265)Resolving deltas:  25% (67/265)Resolving deltas:  26% (69/265)Resolving deltas:  27% (72/265)Resolving deltas:  28% (75/265)Resolving deltas:  29% (77/265)Resolving deltas:  30% (80/265)Resolving deltas:  31% (83/265)Resolving deltas:  32% (85/265)Resolving deltas:  33% (88/265)Resolving deltas:  34% (91/265)Resolving deltas:  35% (94/265)Resolving deltas:  36% (96/265)Resolving deltas:  37% (99/265)Resolving deltas:  38% (101/265)Resolving deltas:  39% (104/265)Resolving deltas:  40% (106/265)Resolving deltas:  41% (109/265)Resolving deltas:  42% (112/265)Resolving deltas:  43% (114/265)Resolving deltas:  44% (117/265)Resolving deltas:  45% (120/265)Resolving deltas:  46% (122/265)Resolving deltas:  47% (125/265)Resolving deltas:  49% (130/265)Resolving deltas:  51% (136/265)Resolving deltas:  52% (139/265)Resolving deltas:  53% (142/265)Resolving deltas:  54% (144/265)Resolving deltas:  55% (146/265)Resolving deltas:  56% (149/265)Resolving deltas:  57% (152/265)Resolving deltas:  58% (154/265)Resolving deltas:  60% (159/265)Resolving deltas:  61% (162/265)Resolving deltas:  62% (165/265)Resolving deltas:  63% (167/265)Resolving deltas:  64% (170/265)Resolving deltas:  65% (173/265)Resolving deltas:  66% (175/265)Resolving deltas:  67% (178/265)Resolving deltas:  68% (181/265)Resolving deltas:  69% (183/265)Resolving deltas:  70% (187/265)Resolving deltas:  71% (190/265)Resolving deltas:  72% (191/265)Resolving deltas:  73% (195/265)Resolving deltas:  74% (197/265)Resolving deltas:  75% (199/265)Resolving deltas:  76% (202/265)Resolving deltas:  77% (206/265)Resolving deltas:  78% (207/265)Resolving deltas:  79% (210/265)Resolving deltas:  80% (213/265)Resolving deltas:  81% (215/265)Resolving deltas:  82% (218/265)Resolving deltas:  83% (220/265)Resolving deltas:  84% (223/265)Resolving deltas:  85% (226/265)Resolving deltas:  86% (229/265)Resolving deltas:  87% (231/265)Resolving deltas:  88% (234/265)Resolving deltas:  89% (236/265)Resolving deltas:  90% (239/265)Resolving deltas:  91% (242/265)Resolving deltas:  92% (244/265)Resolving deltas:  93% (247/265)Resolving deltas:  94% (250/265)Resolving deltas:  95% (252/265)Resolving deltas:  96% (255/265)Resolving deltas:  97% (258/265)Resolving deltas:  98% (260/265)Resolving deltas:  99% (263/265)Resolving deltas: 100% (265/265)Resolving deltas: 100% (265/265), done.
From https://github.com/OnixResearch/nickel-export
 * [new branch]      main       -> main
remote: Enumerating objects: 1165, done.
remote: Counting objects:   0% (1/1165)        remote: Counting objects:   1% (12/1165)        remote: Counting objects:   2% (24/1165)        remote: Counting objects:   3% (35/1165)        remote: Counting objects:   4% (47/1165)        remote: Counting objects:   5% (59/1165)        remote: Counting objects:   6% (70/1165)        remote: Counting objects:   7% (82/1165)        remote: Counting objects:   8% (94/1165)        remote: Counting objects:   9% (105/1165)        remote: Counting objects:  10% (117/1165)        remote: Counting objects:  11% (129/1165)        remote: Counting objects:  12% (140/1165)        remote: Counting objects:  13% (152/1165)        remote: Counting objects:  14% (164/1165)        remote: Counting objects:  15% (175/1165)        remote: Counting objects:  16% (187/1165)        remote: Counting objects:  17% (199/1165)        remote: Counting objects:  18% (210/1165)        remote: Counting objects:  19% (222/1165)        remote: Counting objects:  20% (233/1165)        remote: Counting objects:  21% (245/1165)        remote: Counting objects:  22% (257/1165)        remote: Counting objects:  23% (268/1165)        remote: Counting objects:  24% (280/1165)        remote: Counting objects:  25% (292/1165)        remote: Counting objects:  26% (303/1165)        remote: Counting objects:  27% (315/1165)        remote: Counting objects:  28% (327/1165)        remote: Counting objects:  29% (338/1165)        remote: Counting objects:  30% (350/1165)        remote: Counting objects:  31% (362/1165)        remote: Counting objects:  32% (373/1165)        remote: Counting objects:  33% (385/1165)        remote: Counting objects:  34% (397/1165)        remote: Counting objects:  35% (408/1165)        remote: Counting objects:  36% (420/1165)        remote: Counting objects:  37% (432/1165)        remote: Counting objects:  38% (443/1165)        remote: Counting objects:  39% (455/1165)        remote: Counting objects:  40% (466/1165)        remote: Counting objects:  41% (478/1165)        remote: Counting objects:  42% (490/1165)        remote: Counting objects:  43% (501/1165)        remote: Counting objects:  44% (513/1165)        remote: Counting objects:  45% (525/1165)        remote: Counting objects:  46% (536/1165)        remote: Counting objects:  47% (548/1165)        remote: Counting objects:  48% (560/1165)        remote: Counting objects:  49% (571/1165)        remote: Counting objects:  50% (583/1165)        remote: Counting objects:  51% (595/1165)        remote: Counting objects:  52% (606/1165)        remote: Counting objects:  53% (618/1165)        remote: Counting objects:  54% (630/1165)        remote: Counting objects:  55% (641/1165)        remote: Counting objects:  56% (653/1165)        remote: Counting objects:  57% (665/1165)        remote: Counting objects:  58% (676/1165)        remote: Counting objects:  59% (688/1165)        remote: Counting objects:  60% (699/1165)        remote: Counting objects:  61% (711/1165)        remote: Counting objects:  62% (723/1165)        remote: Counting objects:  63% (734/1165)        remote: Counting objects:  64% (746/1165)        remote: Counting objects:  65% (758/1165)        remote: Counting objects:  66% (769/1165)        remote: Counting objects:  67% (781/1165)        remote: Counting objects:  68% (793/1165)        remote: Counting objects:  69% (804/1165)        remote: Counting objects:  70% (816/1165)        remote: Counting objects:  71% (828/1165)        remote: Counting objects:  72% (839/1165)        remote: Counting objects:  73% (851/1165)        remote: Counting objects:  74% (863/1165)        remote: Counting objects:  75% (874/1165)        remote: Counting objects:  76% (886/1165)        remote: Counting objects:  77% (898/1165)        remote: Counting objects:  78% (909/1165)        remote: Counting objects:  79% (921/1165)        remote: Counting objects:  80% (932/1165)        remote: Counting objects:  81% (944/1165)        remote: Counting objects:  82% (956/1165)        remote: Counting objects:  83% (967/1165)        remote: Counting objects:  84% (979/1165)        remote: Counting objects:  85% (991/1165)        remote: Counting objects:  86% (1002/1165)        remote: Counting objects:  87% (1014/1165)        remote: Counting objects:  88% (1026/1165)        remote: Counting objects:  89% (1037/1165)        remote: Counting objects:  90% (1049/1165)        remote: Counting objects:  91% (1061/1165)        remote: Counting objects:  92% (1072/1165)        remote: Counting objects:  93% (1084/1165)        remote: Counting objects:  94% (1096/1165)        remote: Counting objects:  95% (1107/1165)        remote: Counting objects:  96% (1119/1165)        remote: Counting objects:  97% (1131/1165)        remote: Counting objects:  98% (1142/1165)        remote: Counting objects:  99% (1154/1165)        remote: Counting objects: 100% (1165/1165)        remote: Counting objects: 100% (1165/1165), done.
remote: Compressing objects:   0% (1/814)        remote: Compressing objects:   1% (9/814)        remote: Compressing objects:   2% (17/814)        remote: Compressing objects:   3% (25/814)        remote: Compressing objects:   4% (33/814)        remote: Compressing objects:   5% (41/814)        remote: Compressing objects:   6% (49/814)        remote: Compressing objects:   7% (57/814)        remote: Compressing objects:   8% (66/814)        remote: Compressing objects:   9% (74/814)        remote: Compressing objects:  10% (82/814)        remote: Compressing objects:  11% (90/814)        remote: Compressing objects:  12% (98/814)        remote: Compressing objects:  13% (106/814)        remote: Compressing objects:  14% (114/814)        remote: Compressing objects:  15% (123/814)        remote: Compressing objects:  16% (131/814)        remote: Compressing objects:  17% (139/814)        remote: Compressing objects:  18% (147/814)        remote: Compressing objects:  19% (155/814)        remote: Compressing objects:  20% (163/814)        remote: Compressing objects:  21% (171/814)        remote: Compressing objects:  22% (180/814)        remote: Compressing objects:  23% (188/814)        remote: Compressing objects:  24% (196/814)        remote: Compressing objects:  25% (204/814)        remote: Compressing objects:  26% (212/814)        remote: Compressing objects:  27% (220/814)        remote: Compressing objects:  28% (228/814)        remote: Compressing objects:  29% (237/814)        remote: Compressing objects:  30% (245/814)        remote: Compressing objects:  31% (253/814)        remote: Compressing objects:  32% (261/814)        remote: Compressing objects:  33% (269/814)        remote: Compressing objects:  34% (277/814)        remote: Compressing objects:  35% (285/814)        remote: Compressing objects:  36% (294/814)        remote: Compressing objects:  37% (302/814)        remote: Compressing objects:  38% (310/814)        remote: Compressing objects:  39% (318/814)        remote: Compressing objects:  40% (326/814)        remote: Compressing objects:  41% (334/814)        remote: Compressing objects:  42% (342/814)        remote: Compressing objects:  43% (351/814)        remote: Compressing objects:  44% (359/814)        remote: Compressing objects:  45% (367/814)        remote: Compressing objects:  46% (375/814)        remote: Compressing objects:  47% (383/814)        remote: Compressing objects:  48% (391/814)        remote: Compressing objects:  49% (399/814)        remote: Compressing objects:  50% (407/814)        remote: Compressing objects:  51% (416/814)        remote: Compressing objects:  52% (424/814)        remote: Compressing objects:  53% (432/814)        remote: Compressing objects:  54% (440/814)        remote: Compressing objects:  55% (448/814)        remote: Compressing objects:  56% (456/814)        remote: Compressing objects:  57% (464/814)        remote: Compressing objects:  58% (473/814)        remote: Compressing objects:  59% (481/814)        remote: Compressing objects:  60% (489/814)        remote: Compressing objects:  61% (497/814)        remote: Compressing objects:  62% (505/814)        remote: Compressing objects:  63% (513/814)        remote: Compressing objects:  64% (521/814)        remote: Compressing objects:  65% (530/814)        remote: Compressing objects:  66% (538/814)        remote: Compressing objects:  67% (546/814)        remote: Compressing objects:  68% (554/814)        remote: Compressing objects:  69% (562/814)        remote: Compressing objects:  70% (570/814)        remote: Compressing objects:  71% (578/814)        remote: Compressing objects:  72% (587/814)        remote: Compressing objects:  73% (595/814)        remote: Compressing objects:  74% (603/814)        remote: Compressing objects:  75% (611/814)        remote: Compressing objects:  76% (619/814)        remote: Compressing objects:  77% (627/814)        remote: Compressing objects:  78% (635/814)        remote: Compressing objects:  79% (644/814)        remote: Compressing objects:  80% (652/814)        remote: Compressing objects:  81% (660/814)        remote: Compressing objects:  82% (668/814)        remote: Compressing objects:  83% (676/814)        remote: Compressing objects:  84% (684/814)        remote: Compressing objects:  85% (692/814)        remote: Compressing objects:  86% (701/814)        remote: Compressing objects:  87% (709/814)        remote: Compressing objects:  88% (717/814)        remote: Compressing objects:  89% (725/814)        remote: Compressing objects:  90% (733/814)        remote: Compressing objects:  91% (741/814)        remote: Compressing objects:  92% (749/814)        remote: Compressing objects:  93% (758/814)        remote: Compressing objects:  94% (766/814)        remote: Compressing objects:  95% (774/814)        remote: Compressing objects:  96% (782/814)        remote: Compressing objects:  97% (790/814)        remote: Compressing objects:  98% (798/814)        remote: Compressing objects:  99% (806/814)        remote: Compressing objects: 100% (814/814)        remote: Compressing objects: 100% (814/814), done.
Receiving objects:   0% (1/1165)Receiving objects:   1% (12/1165)Receiving objects:   2% (24/1165)Receiving objects:   3% (35/1165)Receiving objects:   4% (47/1165)Receiving objects:   5% (59/1165)Receiving objects:   6% (70/1165)Receiving objects:   7% (82/1165)Receiving objects:   8% (94/1165)Receiving objects:   9% (105/1165)Receiving objects:  10% (117/1165)Receiving objects:  11% (129/1165)Receiving objects:  12% (140/1165)Receiving objects:  13% (152/1165)Receiving objects:  14% (164/1165)Receiving objects:  15% (175/1165)Receiving objects:  16% (187/1165)Receiving objects:  17% (199/1165)Receiving objects:  18% (210/1165)Receiving objects:  19% (222/1165)Receiving objects:  20% (233/1165)Receiving objects:  21% (245/1165)Receiving objects:  22% (257/1165)Receiving objects:  23% (268/1165)Receiving objects:  24% (280/1165)Receiving objects:  25% (292/1165)Receiving objects:  26% (303/1165)Receiving objects:  27% (315/1165)Receiving objects:  28% (327/1165)Receiving objects:  29% (338/1165)Receiving objects:  30% (350/1165)Receiving objects:  31% (362/1165)Receiving objects:  32% (373/1165)Receiving objects:  33% (385/1165)Receiving objects:  34% (397/1165)Receiving objects:  35% (408/1165)Receiving objects:  36% (420/1165)Receiving objects:  37% (432/1165)Receiving objects:  38% (443/1165)Receiving objects:  39% (455/1165)Receiving objects:  40% (466/1165)Receiving objects:  41% (478/1165)Receiving objects:  42% (490/1165)Receiving objects:  43% (501/1165)Receiving objects:  44% (513/1165)Receiving objects:  45% (525/1165)Receiving objects:  46% (536/1165)Receiving objects:  47% (548/1165)Receiving objects:  48% (560/1165)Receiving objects:  49% (571/1165)Receiving objects:  50% (583/1165)Receiving objects:  51% (595/1165)Receiving objects:  52% (606/1165)Receiving objects:  53% (618/1165)Receiving objects:  54% (630/1165)Receiving objects:  55% (641/1165)Receiving objects:  56% (653/1165)Receiving objects:  57% (665/1165)Receiving objects:  58% (676/1165)Receiving objects:  59% (688/1165)Receiving objects:  60% (699/1165)Receiving objects:  61% (711/1165)Receiving objects:  62% (723/1165)Receiving objects:  63% (734/1165)Receiving objects:  64% (746/1165)Receiving objects:  65% (758/1165)Receiving objects:  66% (769/1165)Receiving objects:  67% (781/1165)Receiving objects:  68% (793/1165)Receiving objects:  69% (804/1165)Receiving objects:  70% (816/1165)Receiving objects:  71% (828/1165)Receiving objects:  72% (839/1165)Receiving objects:  73% (851/1165)Receiving objects:  74% (863/1165)Receiving objects:  75% (874/1165)Receiving objects:  76% (886/1165)Receiving objects:  77% (898/1165)Receiving objects:  78% (909/1165)Receiving objects:  79% (921/1165)Receiving objects:  80% (932/1165)Receiving objects:  81% (944/1165)Receiving objects:  82% (956/1165)Receiving objects:  83% (967/1165)Receiving objects:  84% (979/1165)Receiving objects:  85% (991/1165)Receiving objects:  86% (1002/1165)Receiving objects:  87% (1014/1165)Receiving objects:  88% (1026/1165)Receiving objects:  89% (1037/1165)remote: Total 1165 (delta 246), reused 951 (delta 202), pack-reused 0 (from 0)
Receiving objects:  90% (1049/1165)Receiving objects:  91% (1061/1165)Receiving objects:  92% (1072/1165)Receiving objects:  93% (1084/1165)Receiving objects:  94% (1096/1165)Receiving objects:  95% (1107/1165)Receiving objects:  96% (1119/1165)Receiving objects:  97% (1131/1165)Receiving objects:  98% (1142/1165)Receiving objects:  99% (1154/1165)Receiving objects: 100% (1165/1165)Receiving objects: 100% (1165/1165), 816.86 KiB | 13.39 MiB/s, done.
Resolving deltas:   0% (0/246)Resolving deltas:   1% (3/246)Resolving deltas:   2% (5/246)Resolving deltas:   3% (8/246)Resolving deltas:   4% (10/246)Resolving deltas:   5% (13/246)Resolving deltas:   6% (15/246)Resolving deltas:   7% (18/246)Resolving deltas:   8% (20/246)Resolving deltas:   9% (23/246)Resolving deltas:  10% (25/246)Resolving deltas:  11% (28/246)Resolving deltas:  12% (30/246)Resolving deltas:  13% (32/246)Resolving deltas:  14% (35/246)Resolving deltas:  15% (37/246)Resolving deltas:  16% (41/246)Resolving deltas:  17% (42/246)Resolving deltas:  18% (45/246)Resolving deltas:  19% (47/246)Resolving deltas:  20% (50/246)Resolving deltas:  21% (52/246)Resolving deltas:  22% (55/246)Resolving deltas:  23% (57/246)Resolving deltas:  24% (60/246)Resolving deltas:  25% (63/246)Resolving deltas:  26% (65/246)Resolving deltas:  27% (67/246)Resolving deltas:  28% (69/246)Resolving deltas:  29% (72/246)Resolving deltas:  30% (74/246)Resolving deltas:  31% (77/246)Resolving deltas:  32% (79/246)Resolving deltas:  33% (82/246)Resolving deltas:  34% (84/246)Resolving deltas:  35% (88/246)Resolving deltas:  36% (90/246)Resolving deltas:  37% (92/246)Resolving deltas:  38% (94/246)Resolving deltas:  39% (97/246)Resolving deltas:  40% (100/246)Resolving deltas:  41% (102/246)Resolving deltas:  42% (104/246)Resolving deltas:  43% (106/246)Resolving deltas:  44% (109/246)Resolving deltas:  45% (111/246)Resolving deltas:  46% (114/246)Resolving deltas:  47% (116/246)Resolving deltas:  48% (119/246)Resolving deltas:  49% (121/246)Resolving deltas:  50% (123/246)Resolving deltas:  51% (126/246)Resolving deltas:  52% (128/246)Resolving deltas:  53% (131/246)Resolving deltas:  54% (133/246)Resolving deltas:  56% (138/246)Resolving deltas:  57% (142/246)Resolving deltas:  58% (144/246)Resolving deltas:  59% (146/246)Resolving deltas:  60% (148/246)Resolving deltas:  61% (151/246)Resolving deltas:  62% (153/246)Resolving deltas:  63% (156/246)Resolving deltas:  64% (158/246)Resolving deltas:  65% (161/246)Resolving deltas:  66% (164/246)Resolving deltas:  67% (165/246)Resolving deltas:  68% (169/246)Resolving deltas:  69% (170/246)Resolving deltas:  70% (174/246)Resolving deltas:  71% (175/246)Resolving deltas:  72% (179/246)Resolving deltas:  73% (182/246)Resolving deltas:  74% (183/246)Resolving deltas:  75% (185/246)Resolving deltas:  76% (187/246)Resolving deltas:  77% (190/246)Resolving deltas:  78% (192/246)Resolving deltas:  79% (195/246)Resolving deltas:  80% (198/246)Resolving deltas:  81% (200/246)Resolving deltas:  82% (202/246)Resolving deltas:  83% (205/246)Resolving deltas:  84% (207/246)Resolving deltas:  85% (210/246)Resolving deltas:  86% (213/246)Resolving deltas:  87% (215/246)Resolving deltas:  88% (217/246)Resolving deltas:  89% (219/246)Resolving deltas:  90% (223/246)Resolving deltas:  91% (224/246)Resolving deltas:  92% (227/246)Resolving deltas:  93% (229/246)Resolving deltas:  94% (232/246)Resolving deltas:  95% (234/246)Resolving deltas:  96% (237/246)Resolving deltas:  97% (239/246)Resolving deltas:  98% (242/246)Resolving deltas:  99% (244/246)Resolving deltas: 100% (246/246)Resolving deltas: 100% (246/246), done.
From ssh://github.com/brittonr/tigerstyle-rs
 * branch            bbf5fbb60679668ca8c42593fd617db2d0f89b43 -> FETCH_HEAD
building '/nix/store/5369spi78ikxv4bnf6pg0y4gzz8sjc7n-rustup.drv'...
copying path '/nix/store/xqnwl8qppzhr3dq5nzw2ly6yx4lz9xyr-cargo-deny-0.19.0' from 'https://cache.nixos.org'...
copying path '/nix/store/aiqvnws9qv5xj6kn4qp6wjblasqd8dz2-rust-analyzer-unwrapped-2026-03-30' from 'https://cache.nixos.org'...
copying path '/nix/store/rwc6qkv8qbmcam7sj3lmyi1hapz57wvz-lld-21.1.8-lib' from 'https://cache.nixos.org'...
copying path '/nix/store/2c5r0kpn5yb8qsvfw77j8h63pklij8rq-lld-21.1.8' from 'https://cache.nixos.org'...
copying path '/nix/store/0nl9w8qhb0kvix80vpa191a59mkhw1q9-musl-x86_64-unknown-linux-musl-1.2.5' from 'https://cache.nixos.org'...
copying path '/nix/store/6wyifl152bvm910i7v5pkycfdbng2cav-linux-headers-6.18.7' from 'https://cache.nixos.org'...
copying path '/nix/store/56q4kb4149kw4fi7hrbrrfwgivbqmcfw-x86_64-unknown-linux-musl-gcc-15.2.0-libgcc' from 'https://cache.nixos.org'...
copying path '/nix/store/9d9il9kxwzzy2hpwk3czvjlk7fv15b0j-musl-x86_64-unknown-linux-musl-1.2.5-bin' from 'https://cache.nixos.org'...
copying path '/nix/store/9s5468sc5hciq047wzn7vcg05j05bfa6-mold-unwrapped-2.40.4' from 'https://cache.nixos.org'...
copying path '/nix/store/77rzpdmi3ankx1b605wfa77kkzyhkd6g-x86_64-unknown-linux-musl-gcc-15.2.0-lib' from 'https://cache.nixos.org'...
copying path '/nix/store/mwygqdwl6ysbw4aygi0hy3x0si09nkqn-mold-unwrapped-wrapper-2.40.4' from 'https://cache.nixos.org'...
copying path '/nix/store/1l2lnx0q7smdbwjmccxgicw2g12zvb3q-musl-x86_64-unknown-linux-musl-1.2.5-dev' from 'https://cache.nixos.org'...
copying path '/nix/store/9f2r3vz2lpmhiwgnz08i85zw2m0di9b2-x86_64-unknown-linux-musl-binutils-wrapper-2.44' from 'https://cache.nixos.org'...
copying path '/nix/store/cibbjc8bn3440qq3m6cj846f83iyj5yg-x86_64-unknown-linux-musl-gcc-15.2.0' from 'https://cache.nixos.org'...
copying path '/nix/store/v64aggksnpk4lwng8ivglwcj6qpzvriz-lld-21.1.8-dev' from 'https://cache.nixos.org'...
building '/nix/store/lzy6j8kdl1rlsxv49x4hadiyijgscidk-cargo-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/qsi20c1fmc7p2jy1zdv0xcm4ajmjcpbj-rust-std-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
cannot build on 'ssh-ng://root@10.10.10.1': error: failed to start SSH connection to '10.10.10.1'
building '/nix/store/gj7j2mr5zm93pm6xmnrygfbi4i17llvc-check-nickel-configs.drv'...
copying path '/nix/store/zq24d8np6247dkf3zf6dk7s1y8pr6ivz-ast-grep-0.42.1' from 'https://cache.nixos.org'...
building '/nix/store/358z0ygz6qib9gpgs79sp2llm0d3q2ls-check-store-overlay-policy.drv'...
copying path '/nix/store/ix90ik8pf0ah1vgvpdzg38xvq6njmw2b-wac-cli-0.9.0' from 'https://cache.nixos.org'...
copying path '/nix/store/cmrahwlsdrxgf95ayk4x9yxkf0zb6fwb-wasm-tools-1.245.1' from 'https://cache.nixos.org'...
copying path '/nix/store/psya54xn9znyf7p2c2z42gsj5x5rii4k-wit-bindgen-0.55.0' from 'https://cache.nixos.org'...
copying path '/nix/store/qnc6zp9sr484h2kjsabx6shkbgh6bvzq-wasmtime-43.0.0' from 'https://cache.nixos.org'...
copying path '/nix/store/b14ll8r0naq3rxnf5fx4j0qn0k5s69j1-wizer-10.0.0' from 'https://cache.nixos.org'...
copying path '/nix/store/5lri73xj3vmxcf4jfqa0bh9zcixd2i1d-wkg-0.15.0' from 'https://cache.nixos.org'...
building '/nix/store/j4a95g1gir9qnx2jm1aln4g7aw7vgsv4-llvm-tools-preview-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
copying path '/nix/store/wwkx074y67qzpkk9q6k947jwjwwdybh2-rust-analyzer-2026-03-30' from 'https://cache.nixos.org'...
building '/nix/store/w2fiqsnn0j88y5kd9mzlbz3m72j4v6gc-rust-src-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
copying path '/nix/store/ch1ap98c3cjwzfnzhfjlf1g266ryqdjp-cargo-package-indexmap-2.14.0' from 'https://nix-community.cachix.org'...
copying path '/nix/store/5h68vx3rhh99b01wmriinw6amcix3j0x-cargo-package-find-msvc-tools-0.1.9' from 'https://nix-community.cachix.org'...
copying path '/nix/store/n0c25b706lgzwawq4zablxp21a6yj2ip-cargo-package-errno-0.3.14' from 'https://nix-community.cachix.org'...
copying path '/nix/store/rr34f7188zrc8hgl5w134fh0srviwppn-cargo-package-shlex-1.3.0' from 'https://nix-community.cachix.org'...
copying path '/nix/store/4z969g89hf5zyq4h3sh545ndr3lc2f71-cargo-package-rustix-1.1.4' from 'https://nix-community.cachix.org'...
copying path '/nix/store/9xfhlb6d9mx72d46nbm9di0vxdka3h77-cargo-package-once_cell-1.21.4' from 'https://nix-community.cachix.org'...
copying path '/nix/store/r90zxs406fqhh327q0dqfi8zllfsi2yd-cargo-package-tempfile-3.27.0' from 'https://nix-community.cachix.org'...
copying path '/nix/store/l1zfql96if5aal9qy5v6sxj7khi74sd6-cargo-package-rustversion-1.0.22' from 'https://nix-community.cachix.org'...
copying path '/nix/store/xlkmhnygxwn784sfj96nz3jmrvarj5kg-cargo-package-anyhow-1.0.102' from 'https://nix-community.cachix.org'...
copying path '/nix/store/bmlnl60d4v97dsrkd5a563kwvasshn77-cargo-package-windows-link-0.2.1' from 'https://nix-community.cachix.org'...
copying path '/nix/store/kz8w1lprk5bw8m3bq4ci4krlz9kk9y0h-cargo-package-serde_core-1.0.228' from 'https://nix-community.cachix.org'...
copying path '/nix/store/6258xisac4877aahq8iz30kqdsnii3gr-cargo-package-serde_derive-1.0.228' from 'https://nix-community.cachix.org'...
copying path '/nix/store/18l7x2yn94nxxk70plc7cplj788qaafp-cargo-package-aho-corasick-1.1.4' from 'https://nix-community.cachix.org'...
copying path '/nix/store/cs8iz41g6wdvxs8xjkhq71fa3w8qvl3k-cargo-package-log-0.4.29' from 'https://nix-community.cachix.org'...
copying path '/nix/store/vblddz583fycy89mhsybc2k9v90ci84l-cargo-package-syn-2.0.117' from 'https://nix-community.cachix.org'...
copying path '/nix/store/vfl5x2xk1cnc7savc4ibxkvk88als607-cargo-package-windows-sys-0.61.2' from 'https://nix-community.cachix.org'...
copying path '/nix/store/kl616vdzd1gdisl11h97yf4v3dfbyx4m-cargo-package-proc-macro2-1.0.106' from 'https://nix-community.cachix.org'...
copying path '/nix/store/k8ndg0x4i7absnghvlknaqsmh7dah9km-cargo-package-serde-1.0.228' from 'https://nix-community.cachix.org'...
copying path '/nix/store/8d4jy2m41jbl13710j1d6bk7ssxfi3rr-rust-docs-1.90.0-x86_64-unknown-linux-gnu' from 'https://nix-community.cachix.org'...
copying path '/nix/store/pkkmdya4xylfyz1f4kk6d7a2xhkzj1lp-cargo-package-regex-1.12.3' from 'https://nix-community.cachix.org'...
copying path '/nix/store/8x2mznhaa4a4ri28xnmwr7vxsmzqzp7f-cargo-package-cfg-if-1.0.4' from 'https://nix-community.cachix.org'...
copying path '/nix/store/bzx4bn0sz84sdnws9ibsx10cxkig8w68-cargo-package-itoa-1.0.18' from 'https://nix-community.cachix.org'...
copying path '/nix/store/jdcppwcwpw6cnqdkds8cvwj7i44n3g75-cargo-package-regex-syntax-0.8.10' from 'https://nix-community.cachix.org'...
copying path '/nix/store/skbh9jsvx8kjx24w9yxiihx4hv3ahyzr-cargo-package-equivalent-1.0.2' from 'https://nix-community.cachix.org'...
copying path '/nix/store/q2fqcbq5a2ya5g1v6jjpqnrrwgyaaz0n-cargo-package-serde_json-1.0.149' from 'https://nix-community.cachix.org'...
copying path '/nix/store/y5bz7dr7l4n4jz320x8y152f05mmwr4g-cargo-package-wit-bindgen-0.51.0' from 'https://nix-community.cachix.org'...
copying path '/nix/store/52507hqil11ad3c4a37qw1mw8afqf0cs-cargo-package-hashbrown-0.17.0' from 'https://nix-community.cachix.org'...
copying path '/nix/store/4m9walqj51vl13yx6hfc9i44m1gmgskz-cargo-package-linux-raw-sys-0.12.1' from 'https://nix-community.cachix.org'...
copying path '/nix/store/1gh1r8lbzp5c12zcmc3pizrc0q5g7fgs-cargo-package-memchr-2.8.0' from 'https://nix-community.cachix.org'...
copying path '/nix/store/wyc363w93c3cby1v6rqbqz141lff38iq-cargo-package-quote-1.0.45' from 'https://nix-community.cachix.org'...
copying path '/nix/store/sik4ijq58kqxijfi0h22gwl8cq6911zr-cargo-package-regex-automata-0.4.14' from 'https://nix-community.cachix.org'...
copying path '/nix/store/xrjia7g04xcnkyyll2rv2lhndh5wg0yn-cargo-package-semver-1.0.28' from 'https://nix-community.cachix.org'...
copying path '/nix/store/rjk1hhv124bc163z7al9rnyc5aspv88h-cargo-package-unicode-ident-1.0.24' from 'https://nix-community.cachix.org'...
copying path '/nix/store/lh8bi5jg7ck3ka4f6pgcw5i4kyzy4wgd-cargo-package-zmij-1.0.21' from 'https://nix-community.cachix.org'...
building '/nix/store/hwz43xm09gs5v38a4pfyc2dhs5n9h1pj-mantle-ast-grep-toolchain-0.42.1.drv'...
building '/nix/store/dpmmaf4vi36jf5wj1ryjpq8mfj5axc07-cargo-package-yoke-derive-0.8.1.drv'...
building '/nix/store/d5fn2bx7x5r5ja68lysqnfhylpp7yq2l-cargo-package-windows_x86_64_msvc-0.52.6.drv'...
building '/nix/store/d5ss57ww37fgim8sg3hc7mlcqk4vljcd-cargo-package-toml_parser-1.1.2+spec-1.1.0.drv'...
building '/nix/store/fhc67cmhpfiy0brs1msln6f40d11mdc6-cargo-package-winnow-0.7.14.drv'...
copying path '/nix/store/zalq324g2waz1i722xn94i4d2cmzd4g6-cargo-src-libc-0.2.185' from 'https://nix-community.cachix.org'...
building '/nix/store/67lqjzglaj8lb5mdssl03bh583ghjzdd-cargo-git-ssh-git-github.com-OnixResearch-valence.git-1f4f6fe903a34c22d2cda53cd534ce58bbf84147.drv'...
building '/nix/store/dpi8mm5c05280ys737idns3rvhsvrs57-cargo-package-zerofrom-0.1.6.drv'...
building '/nix/store/gzwaayd4f4hiwnd7q75q4ga99s7jgj6x-cargo-package-zerovec-0.11.5.drv'...
copying path '/nix/store/y146mxs1bfivrbnwvkjgj9gy1kn98hb3-adler2-2.0.0' from 'https://cache.nixos.org'...
copying path '/nix/store/2knnkv3lx4p6xxlb90frzgilrzf6lwm6-ahash-0.8.11' from 'https://cache.nixos.org'...
copying path '/nix/store/r94d91v89f6xmv298q0klnmw3yr9k8hm-anstyle-parse-0.2.6' from 'https://cache.nixos.org'...
copying path '/nix/store/3hpl9i2lxmd2wfl2caxldrrqgjf43xr1-anstyle-1.0.10' from 'https://cache.nixos.org'...
copying path '/nix/store/mlw2l9y4kpfpkn0k3wr7arxrhdk9vyzq-arbitrary-1.4.1' from 'https://cache.nixos.org'...
copying path '/nix/store/b4d1nhahdj5fx31ckc95p556dy11w5pz-anstream-0.6.18' from 'https://cache.nixos.org'...
copying path '/nix/store/mg1yhrs3n6flr8dj6gzhg8g3mmj52qpc-anstyle-query-1.1.2' from 'https://cache.nixos.org'...
copying path '/nix/store/c9l0njb87czv8fqv3gb08lz7xz45a61g-bumpalo-3.16.0' from 'https://cache.nixos.org'...
copying path '/nix/store/5bh69fk10n11xhkhwfkk0yldzq1k72n3-cap-primitives-3.4.4' from 'https://cache.nixos.org'...
copying path '/nix/store/qzmjyyj21k7ncv7vdh0nhs6hgghxzklr-autocfg-1.4.0' from 'https://cache.nixos.org'...
copying path '/nix/store/cdjhkrrnbyyf1119rsdj01wrfw59g0h7-backtrace-0.3.74' from 'https://cache.nixos.org'...
copying path '/nix/store/kawydaq9qlmjchmk7lkdjdxhnb9mdhg0-bitmaps-2.1.0' from 'https://cache.nixos.org'...
copying path '/nix/store/vx1sw46fcprazr6v9mjy957msrw6k2np-bytes-1.10.1' from 'https://cache.nixos.org'...
copying path '/nix/store/hvdx8ln3vrqkbn359zg7195hfhjwplnp-anstyle-wincon-3.0.6' from 'https://cache.nixos.org'...
building '/nix/store/0lq0zw89x6qzjzqgyn5jjgysi0d7f22r-cargo-package-cpufeatures-0.3.0.drv'...
copying path '/nix/store/jbsjrq8f6bf1ss21vlzxfwfzfapmzf5n-async-trait-0.1.88' from 'https://cache.nixos.org'...
copying path '/nix/store/p9rpf6pp08476jssys9s44643cgbcdjf-bitflags-2.6.0' from 'https://cache.nixos.org'...
copying path '/nix/store/3vchg70kcl653vc4h7v5szgsrn8pq0dn-colorchoice-1.0.3' from 'https://cache.nixos.org'...
copying path '/nix/store/vq6mcfg2956lkzhjigpj7mvb27jx2zaf-cap-std-3.4.4' from 'https://cache.nixos.org'...
copying path '/nix/store/cs99d7sl208bmcbzzkpk7hhm64p03019-cfg-if-1.0.0' from 'https://cache.nixos.org'...
copying path '/nix/store/rpyr8lp0824qcnxqgqbvrph6hfg7wagv-clap_lex-0.7.4' from 'https://cache.nixos.org'...
copying path '/nix/store/fxpshplqfadz8pbgva5m9b8fdjprir2h-clap_builder-4.5.48' from 'https://cache.nixos.org'...
copying path '/nix/store/jv5i3mik2j3ffj25vlnxf71p5s3bgk9m-clap-4.5.48' from 'https://cache.nixos.org'...
copying path '/nix/store/wkjvdr824xgvwinsdx4yybzkhg12mfx1-x86_64-unknown-linux-musl-gcc-wrapper-15.2.0' from 'https://cache.nixos.org'...
copying path '/nix/store/kx5xbm3l327nylq1v7z6y737q4ng9ykf-crypto-common-0.1.6' from 'https://cache.nixos.org'...
copying path '/nix/store/v0hf1bs9523wriswjqgndfz1i59y1hh0-crc32fast-1.4.2' from 'https://cache.nixos.org'...
copying path '/nix/store/9930d3rxmd6v2wg78f4ai9ap7v8jd3qq-dirs-sys-next-0.1.2' from 'https://cache.nixos.org'...
copying path '/nix/store/j8533asq29z1yz6h4rhsj4lhxsllh6hi-env_filter-0.1.3' from 'https://cache.nixos.org'...
copying path '/nix/store/i350fbs8alg3m2b60sb3f5vimcgnrvvl-equivalent-1.0.1' from 'https://cache.nixos.org'...
copying path '/nix/store/1gflm490831a9zlgrmrjs826py3rh757-errno-0.3.10' from 'https://cache.nixos.org'...
building '/nix/store/54mj6x60av7j1gnw1yly64rircyfaybl-cargo-package-regex-syntax-0.8.8.drv'...
copying path '/nix/store/152lrmgbakwimhsq5wcfy69ikfg5avab-fallible-iterator-0.2.0' from 'https://cache.nixos.org'...
copying path '/nix/store/52r4wy34bqqnkrkpq2545baz6dymcvmf-fastrand-2.1.1' from 'https://cache.nixos.org'...
copying path '/nix/store/2gvn8mkimjj3cqdawnw8zf4m71rp0z3l-fd-lock-4.0.4' from 'https://cache.nixos.org'...
copying path '/nix/store/v75nknp8kfxv4livspg3r2nqisjgp7mg-futures-task-0.3.31' from 'https://cache.nixos.org'...
copying path '/nix/store/zkmpcw1wzvjf4z4v9gvawx30hzndclff-futures-0.3.31' from 'https://cache.nixos.org'...
copying path '/nix/store/qrbgnqf162izn2fcjzv7hck5sf7x6qka-futures-channel-0.3.31' from 'https://cache.nixos.org'...
copying path '/nix/store/4inpiik3bhaziqi633dyn9dsvg7ppjr6-futures-sink-0.3.31' from 'https://cache.nixos.org'...
copying path '/nix/store/pby5hyb0vq35sn2rbw124vzfdl6hjynv-form_urlencoded-1.2.1' from 'https://cache.nixos.org'...
copying path '/nix/store/4zxrjknv9ahgywksxwa1qc3bq1hg7i10-getrandom-0.2.15' from 'https://cache.nixos.org'...
copying path '/nix/store/zx4hx55vm12ypp5p62j6sklsr164rcxc-getrandom-0.3.2' from 'https://cache.nixos.org'...
copying path '/nix/store/lijv85f9vdbay8h2g4npmkw97w7zmxg1-hashbrown-0.15.2' from 'https://cache.nixos.org'...
copying path '/nix/store/d2h1bh2i9vqk79sdb10g4pnspm73jwcz-icu_collections-1.5.0' from 'https://cache.nixos.org'...
copying path '/nix/store/lcpa4j97pnryxknmfmq3sijli137fwx3-futures-util-0.3.31' from 'https://cache.nixos.org'...
copying path '/nix/store/58yvv9wzq6lpb4xhkjy55iaa4gyr7xq2-icu_locid-1.5.0' from 'https://cache.nixos.org'...
copying path '/nix/store/mwfh3rhjz5dk3xdxjxpyzbd4wc8jb7fz-icu_locid_transform-1.5.0' from 'https://cache.nixos.org'...
building '/nix/store/qjxgpf7d656zqhnmd4xjpsdsnmk4d3sw-cargo-package-pin-project-lite-0.2.17.drv'...
building '/nix/store/5x94sv8k8mpphvjdar9crh01qfb873pm-cargo-package-quote-1.0.44.drv'...
copying path '/nix/store/bcsb3wyxwibxvwi7ls5bic7pxszki6nf-icu_locid_transform_data-1.5.0' from 'https://cache.nixos.org'...
copying path '/nix/store/kny8irc25bslw0p2vyl0qr51bmsgggq2-icu_normalizer-1.5.0' from 'https://cache.nixos.org'...
copying path '/nix/store/4fvgh9prgrz8r6nspmc1zn3mkp6v4fi3-icu_properties-1.5.1' from 'https://cache.nixos.org'...
copying path '/nix/store/3bgja4f9mqyyd2m7bjm4vlw145rard36-icu_normalizer_data-1.5.0' from 'https://cache.nixos.org'...
copying path '/nix/store/lkf7ab2q0arbrb665678rk7vv59m0wib-icu_properties_data-1.5.0' from 'https://cache.nixos.org'...
copying path '/nix/store/vl0qnadbmngmq7bq2mha8ykzqnkqcnjj-io-uring-0.7.9' from 'https://cache.nixos.org'...
copying path '/nix/store/ghm5sqrdv6npgykbrxpcw2skhrw4g3h8-icu_provider_macros-1.5.0' from 'https://cache.nixos.org'...
copying path '/nix/store/5f2nawhmiplzvbhvgr5ya6p9pjc5kyry-idna-1.0.3' from 'https://cache.nixos.org'...
copying path '/nix/store/0pbn7a7i44m0axx3k0wn8is2d1y2nrn7-idna_adapter-1.2.0' from 'https://cache.nixos.org'...
copying path '/nix/store/f8pbnig5hk76b2ip8g2bjb2mpssyhi01-io-extras-0.18.4' from 'https://cache.nixos.org'...
copying path '/nix/store/6bjsw3f545p0hmwkpy6i0i21mb3hnssc-icu_provider-1.5.0' from 'https://cache.nixos.org'...
building '/nix/store/4c931i0qr5am8d4ifrvb7mr59gn4a3px-cargo-package-crossbeam-queue-0.3.12.drv'...
copying path '/nix/store/87civykyri01kqjpqm0a3xaknss24mjf-codespan-reporting-0.11.1' from 'https://nix-community.cachix.org'...
copying path '/nix/store/x43rgv3x7vzz8x494yz5iw3irplmagia-is_terminal_polyfill-1.70.1' from 'https://cache.nixos.org'...
copying path '/nix/store/6fkjqds1dhhs5p32hbzralj6wcba2l1g-ipnet-2.11.0' from 'https://cache.nixos.org'...
copying path '/nix/store/pvrpjpmq1g0wkh29c71z9fq1plf89s2i-itoa-1.0.14' from 'https://cache.nixos.org'...
copying path '/nix/store/6p0224axy7i31mqvsakxysz1wl82bq47-libredox-0.1.3' from 'https://cache.nixos.org'...
copying path '/nix/store/xczyacayh4bmczjsg7mv2dm9lswpqb7z-js-sys-0.3.77' from 'https://cache.nixos.org'...
copying path '/nix/store/k2mzsh0pa5m2j9435idfvw7dn3680zk2-leb128-0.2.5' from 'https://cache.nixos.org'...
copying path '/nix/store/lqvcbrlr3zr0qjwg74g16a5ywanly1ib-libc-0.2.174' from 'https://cache.nixos.org'...
copying path '/nix/store/mls0887r8i2i5yn2nzar8h0m22c2m1sg-libm-0.2.11' from 'https://cache.nixos.org'...
copying path '/nix/store/9z55q75qrnphwf11k9n76bklfxhxn8jx-jobserver-0.1.32' from 'https://cache.nixos.org'...
copying path '/nix/store/fh57gkgdzkdd2crdw1k07cas2pb9jig0-linux-raw-sys-0.4.14' from 'https://cache.nixos.org'...
copying path '/nix/store/sy449nhibjrxphn636zp1ss4idyq778x-litemap-0.7.5' from 'https://cache.nixos.org'...
building '/nix/store/7qpdrbqfnjsdcz2jmzj5r71ysqa79vbg-cargo-package-paste-1.0.15.drv'...
copying path '/nix/store/6psc52s62ligjwas2wsyysp547vrwsag-indexmap-2.7.1' from 'https://nix-community.cachix.org'...
copying path '/nix/store/gvykcccxsm92n69kv7fbdvwck4yi5bzl-log-0.4.28' from 'https://cache.nixos.org'...
copying path '/nix/store/iqwp1x6ji5wxjd898lv4rp21qczkkdr3-memchr-2.7.4' from 'https://cache.nixos.org'...
copying path '/nix/store/76kf3369svmwksqnfa5qzxb1mzna5zsj-miniz_oxide-0.8.5' from 'https://cache.nixos.org'...
copying path '/nix/store/w36szdnkspkild66aqhzi064szml044k-mio-1.0.3' from 'https://nix-community.cachix.org'...
copying path '/nix/store/l49amlr71a1imh30kfr9x614g9j5ghw9-once_cell-1.19.0' from 'https://cache.nixos.org'...
copying path '/nix/store/in0qrj4iqqgh7kglzscf24vniaamcc2l-object-0.36.7' from 'https://cache.nixos.org'...
copying path '/nix/store/28zq1axqp9yfk2c44x9nyprz7pczar06-percent-encoding-2.3.1' from 'https://cache.nixos.org'...
copying path '/nix/store/wc8y73yjmcjknwyd0s1w5ayc1fnrvfv9-proc-macro2-1.0.94' from 'https://cache.nixos.org'...
copying path '/nix/store/16vfqvq5l9323whxvh37k4fijfnwyig2-pin-project-lite-0.2.14' from 'https://cache.nixos.org'...
copying path '/nix/store/j1qkwrm6zph86pn58mdcpsls5jr5mfkr-pin-utils-0.1.0' from 'https://cache.nixos.org'...
copying path '/nix/store/xnrr8flmbxzsj29jdmm4qqjdv6yv5bzs-ppv-lite86-0.2.20' from 'https://cache.nixos.org'...
building '/nix/store/gbzpp69pdpwz3m02lvnlrb4wfk1hpgpc-cargo-package-libgit2-sys-0.18.3+1.9.2.drv'...
building '/nix/store/bg8p3ahrzc0kcyaavigl38q1cpaiqh0z-cargo-package-tokio-macros-2.7.0.drv'...
copying path '/nix/store/piznn7qsfvfnhwwg3ajsxf1ssngzqn1y-crate-directories-next-2.0.0.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/8yi9y5w1jm3aq6vbsbjaxd6gb5d6cqj2-quote-1.0.37' from 'https://cache.nixos.org'...
copying path '/nix/store/7sm3ydv7rj50naki4yfxwchzk6bcgki3-rand_core-0.9.3' from 'https://cache.nixos.org'...
copying path '/nix/store/lqh1x22vl2ixagjbv1i6c1b7dlw5f8gh-rand-0.9.2' from 'https://cache.nixos.org'...
copying path '/nix/store/92kyzwl29n7znzl9ps4cvxlj464ixjam-rand_xoshiro-0.6.0' from 'https://cache.nixos.org'...
copying path '/nix/store/h9w0swg59v9bys67z9b8552an1dvxf5h-redox_users-0.4.6' from 'https://cache.nixos.org'...
copying path '/nix/store/al3jkl3m2852r4ik8qj3hxnkvbp9wgwc-rustc-hash-2.1.1' from 'https://cache.nixos.org'...
copying path '/nix/store/8p5ki5xna5bmnjpp54rzppzjnd9hbq23-rustc-demangle-0.1.24' from 'https://cache.nixos.org'...
copying path '/nix/store/dibac0jxqg416zknyw4hsv1asgr9jy2i-rustversion-1.0.17' from 'https://cache.nixos.org'...
copying path '/nix/store/gg784i2ngzn71dji7jnxdwnrbm58giy7-ryu-1.0.18' from 'https://cache.nixos.org'...
copying path '/nix/store/5p5z7iz5lf3ph2cw7n90fyxn1m3bdprq-sha2-0.10.8' from 'https://cache.nixos.org'...
copying path '/nix/store/gbdkcc4z283qhlzdlwx0svyxi02x68sh-sized-chunks-0.6.5' from 'https://cache.nixos.org'...
building '/nix/store/bf4hiw4ak3y8xnc4wg8vgjg0p3pczw06-cargo-package-windows_i686_gnullvm-0.53.1.drv'...
copying path '/nix/store/d5dmymmkvinanrz4wh8rsrn1ysv5qmhd-slab-0.4.10' from 'https://cache.nixos.org'...
copying path '/nix/store/sy87l7sffhrnbkpg7j81gp81wkzlq03n-smallvec-1.13.2' from 'https://cache.nixos.org'...
copying path '/nix/store/pd7c8pz8mx0szlijx1iffxfv2hixxbj0-strum-0.24.1' from 'https://cache.nixos.org'...
copying path '/nix/store/fnkq4lsk5lflr8mkgqgmbcwfmmc56jlq-stable_deref_trait-1.2.0' from 'https://cache.nixos.org'...
copying path '/nix/store/0px994rc8kpsrz61hla1dxh9l4710b6b-socket2-0.6.0' from 'https://cache.nixos.org'...
copying path '/nix/store/xzsmaklhykh7l0iii7nji277n54wwps1-strum_macros-0.24.3' from 'https://cache.nixos.org'...
copying path '/nix/store/2jbr73mbhsk5lcwkf58sarcb3khizkxy-syn-2.0.100' from 'https://cache.nixos.org'...
copying path '/nix/store/xybb0w1qx8qrvxxcadajc5xkkpgz62pz-synstructure-0.13.1' from 'https://cache.nixos.org'...
copying path '/nix/store/ppgb8nj701bfv3zsc32sm4bjdqraxz3w-system-interface-0.27.3' from 'https://cache.nixos.org'...
copying path '/nix/store/n2w4423yd3aid6ghpjxqla4n4rdxxl0g-tokio-1.47.1' from 'https://cache.nixos.org'...
copying path '/nix/store/g120zab12fy1ym36y1azr1iczxia25ji-thiserror-1.0.63' from 'https://cache.nixos.org'...
copying path '/nix/store/9iq4frdjh58hy5ajg14k7pagbafbd2zc-thiserror-impl-1.0.63' from 'https://cache.nixos.org'...
copying path '/nix/store/6g1rxj3y1mzfcmfbx1sxmxqrw7w3177z-tinystr-0.7.6' from 'https://cache.nixos.org'...
copying path '/nix/store/8b3fdnihnj2hb4zsgypc63847braqizh-target-lexicon-0.13.2' from 'https://cache.nixos.org'...
building '/nix/store/cf3q12g4zraj6ws0gwxxbbvv2xrcaz5x-cargo-package-unicode-ident-1.0.22.drv'...
copying path '/nix/store/0zlcahax4987y4h8gk1ccyjnb0pkq0a9-tokio-macros-2.5.0' from 'https://cache.nixos.org'...
copying path '/nix/store/l0nlr4bsa6ff8rc562r14c8fqiqig06x-tracing-0.1.41' from 'https://cache.nixos.org'...
copying path '/nix/store/wiqam0kffbgsj52q3yqal4hqdh2zkgvj-tracing-attributes-0.1.28' from 'https://cache.nixos.org'...
copying path '/nix/store/00lzsyl8fz603mdw1k3f6sj0bpv4qcgd-tracing-core-0.1.33' from 'https://cache.nixos.org'...
copying path '/nix/store/d02akxc15jfd7w7kkkyl4vs86cyd7mh4-trait-variant-0.1.2' from 'https://cache.nixos.org'...
copying path '/nix/store/ayckbg26iqfpsvk9xcxm8v7hiil9cn6g-typenum-1.17.0' from 'https://cache.nixos.org'...
copying path '/nix/store/dfwgab23hilzxw2sg111m10qrqacr9q8-unicode-ident-1.0.13' from 'https://cache.nixos.org'...
copying path '/nix/store/7lc5dcfz1577cr3wdns4lc01wj58wk6b-unicode-width-0.1.14' from 'https://cache.nixos.org'...
copying path '/nix/store/hlpfaxzdw0pr1z5n3f3apcfjnvzwcg0b-unicode-width-0.2.0' from 'https://cache.nixos.org'...
copying path '/nix/store/hchrhj7v717fig17mwhjlxfnx36n4bfh-url-2.5.4' from 'https://cache.nixos.org'...
copying path '/nix/store/iwjk0blfk7p73k7dpz11ilsxwwdn7c7k-utf16_iter-1.0.5' from 'https://cache.nixos.org'...
building '/nix/store/d88646dxpl0dnqxjk40cg3p4lqgf0gfv-cargo-package-synstructure-0.13.2.drv'...
copying path '/nix/store/w3b1gqqf16x2y05b8yqf9z6nr6amryq6-crate-addr2line-0.24.1.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/2qklsgfjl7zswh5wfrhvadkbsv4m18yw-crate-cc-1.2.17.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/adxq684zz82m80jc121dsdwzc315fpnj-wasm-bindgen-0.2.100' from 'https://cache.nixos.org'...
copying path '/nix/store/kshgv27a9aiam8rcp3p97dq931mf75sd-wasi-0.11.0+wasi-snapshot-preview1' from 'https://cache.nixos.org'...
copying path '/nix/store/1c9awn9zc1g6kx7ivj0g38ihn1axhqvq-wasi-0.14.2+wasi-0.2.4' from 'https://cache.nixos.org'...
copying path '/nix/store/s4m1rmsgd05f63qhgmk5fnsj61fixgpi-crate-cobs-0.2.3.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/i200pza8fw0dx8vwknf347f4l5x2x09h-wasm-bindgen-backend-0.2.100' from 'https://cache.nixos.org'...
copying path '/nix/store/w41zk3nsg4la68zbrp4nnb1disvk96m0-wasm-bindgen-macro-0.2.100' from 'https://cache.nixos.org'...
copying path '/nix/store/pksxcicx9dfam9wr2v2bidbkzcvlbar8-wasm-bindgen-macro-support-0.2.100' from 'https://cache.nixos.org'...
copying path '/nix/store/9brl6fin7lhy8pc0czc3sqshl18w42qs-wasm-bindgen-shared-0.2.100' from 'https://cache.nixos.org'...
copying path '/nix/store/av4r4d8c7rjz0y1imncp200kr9d60fvz-winapi-util-0.1.9' from 'https://cache.nixos.org'...
copying path '/nix/store/g4951n8nrlc6j7j31bvvsq7kfbv899l1-wit-bindgen-rt-0.39.0' from 'https://cache.nixos.org'...
copying path '/nix/store/7705br74qsv4v3qg3z9nnjprgqdqhiqb-write16-1.0.0' from 'https://cache.nixos.org'...
copying path '/nix/store/036mrr198dsxdxlrqq3wxckr3nwy3jcy-windows-core-0.52.0' from 'https://cache.nixos.org'...
building '/nix/store/pvflmjcnxza68bxryl9chsf60k5awsfi-cargo-package-r-efi-5.3.0.drv'...
building '/nix/store/2svfxwvz5y57pmvlmsmsmggl31km2w94-cargo-package-zmij-1.0.16.drv'...
copying path '/nix/store/58fv0ikf4q8hfd2p03rkjiqnymj8xl3m-writeable-0.5.5' from 'https://cache.nixos.org'...
copying path '/nix/store/x3khqf31qfyfkwlhf627j2cq21dvivbn-yoke-derive-0.7.5' from 'https://cache.nixos.org'...
copying path '/nix/store/6w3fll2ds6dvkwg2ys3rb7jz3l275rg6-zerocopy-0.7.35' from 'https://cache.nixos.org'...
copying path '/nix/store/rlyqk2bh5pvhw3g1hnvzddx73mp1jhzd-yoke-0.7.5' from 'https://cache.nixos.org'...
copying path '/nix/store/kxdl1mwqnym7zwfwa25gnbigjv926cs3-zerocopy-derive-0.7.35' from 'https://cache.nixos.org'...
copying path '/nix/store/32lahaa1sh261i5pbszvxj7i5kl558ci-zerovec-0.10.4' from 'https://cache.nixos.org'...
copying path '/nix/store/r9gqrhkda2h4iv53agnb8ixs8pfqiygy-zerovec-derive-0.10.3' from 'https://cache.nixos.org'...
copying path '/nix/store/6ivz8jdqdxzx93fx2iw7bd3zqj5vhjq3-zstd-sys-2.0.15+zstd.1.5.7' from 'https://cache.nixos.org'...
building '/nix/store/2xwhg3d1bb5hr5dik86dj6c551jigpx8-cargo-package-curl-0.4.49.drv'...
copying path '/nix/store/6sdc1yf8b9znjwqgx6559ffxm57xpcwq-crate-gimli-0.31.0.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/88506dwj82m0din7am2240jfsqj1lrx2-crate-id-arena-2.2.1.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/nhp1r86blx9bgg71kgwalxr2mlzl1mz5-crate-linux-raw-sys-0.9.3.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/6276y2glp4lwzadsar719l316psakx4r-crate-mach2-0.4.2.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/hs32sixafxgc3qr9cvxr3xvs6rvh0f1j-crate-memfd-0.6.4.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/hjq3vl5km0vbsgv54kdphk0ncj0ilj8p-crate-postcard-1.1.1.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/xldsq8nrqhg7xallkggp2cwmf8hnra4b-crate-prettyplease-0.2.25.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/71vgk559fi3p1x3f2dpq4jbbyn2n5qcm-crate-psm-0.1.25.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/95rlbqxr58kb8b0i4knv5jppp3ygcicy-crate-terminal_size-0.4.2.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/kb44jqgqjd13sh290h6k5c5asiwdczq4-crate-serde_json-1.0.128.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/sy5kyi82q91b1i3jpx1cy5cixl4di1b7-crate-sptr-0.3.2.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/618fkmy19wg1qar5lvcdx59a6r2rpqyw-crate-scratch-1.0.7.tar.gz' from 'https://cache.nixos.org'...
building '/nix/store/svxkzs1b5kya1hz5hz5g3pzfs9sdhn98-cargo-package-itoa-1.0.17.drv'...
copying path '/nix/store/7j7wvp7m2qk00b7jvjm2iiap1dhzm17z-crate-clap_derive-4.5.47.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/8jjvmy626lm4qbqv315gmbp9lxcgrjj2-crate-tempfile-3.12.0.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/43z3z0yddlylgz0kgal8wvqw8j2jfhbc-crate-wasm-opt-cxx-sys-0.116.0.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/qv9xis7axnkig0w0lr97zvz3aci92a51-crate-wasm-opt-sys-0.116.0.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/kaa8h8xr2v6q62j32w91vxs2qzp574nj-crate-gimli-0.26.2.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/9wwa0qkkkdafr0fnqjfv0cd7dx28b18y-crate-link-cplusplus-1.0.9.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/qljpzylsl5pg18l885hb22kchj3k397k-crate-rustix-1.0.3.tar.gz' from 'https://cache.nixos.org'...
copying path '/nix/store/qd6xb11bql5i9l89njg2wwl09iv9cl9g-crate-winnow-0.7.11.tar.gz' from 'https://cache.nixos.org'...
building '/nix/store/lv43c7j8cs0vxki8cfykixf9y2m8632i-cargo-package-crossbeam-0.8.4.drv'...
building '/nix/store/zs85kkyszwb8xdy7ij4ri5i12qqwzh3c-rustc-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/knm4dymmy5rglq0vbkiqim481cm023mi-rust-docs-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/5xcy1hk1sxqx856dy4ak2pzy6rjz1523-cargo-package-icu_locale_core-2.1.1.drv'...
building '/nix/store/5qwrai89z5lv750i37laniy0rjnccbv3-rust-docs-nightly-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/sz2r9fcslyz05akkcknr2cm9jahq7753-crate-wasmtime-jit-icache-coherence-30.0.2.tar.gz.drv'...
building '/nix/store/3dq05lawa4dd768sc493w805x529354m-crate-cap-fs-ext-3.4.2.tar.gz.drv'...
building '/nix/store/6vxidcf7ii8gg98c8j4zyfn4rgn4h1j9-wasm-opt-sys-0.116.0.drv'...
building '/nix/store/2fn7mp28480mgmdy7impf1k0gbl6nddk-crate-cxxbridge-macro-1.0.128.tar.gz.drv'...
building '/nix/store/cjslqbs2a9c63pk4pnml5h73fy62d1fd-crate-cranelift-frontend-0.117.2.tar.gz.drv'...
building '/nix/store/h6k7ilydd2h782nvzslyl0aw10m0msz3-crate-cap-time-ext-3.4.2.tar.gz.drv'...
building '/nix/store/pn8hhgcqnp1a7z0ayn003r2pxdz2phkz-crate-wit-bindgen-0.32.0.tar.gz.drv'...
building '/nix/store/ih6zylkwxv6mb1kzj47p1ls65pbqayxa-crate-wasmtime-slab-30.0.2.tar.gz.drv'...
building '/nix/store/1m5h3q7y54pfb9vggijgfdc87s4shp91-crate-wasmtime-wasi-io-30.0.2.tar.gz.drv'...
building '/nix/store/1g4nblcvsn1310nx5pmldpn8llyhspm7-crate-wasmtime-wit-bindgen-30.0.2.tar.gz.drv'...
building '/nix/store/j23c343ji345b154pqm9d4ch7cgx0fi8-crate-wasmtime-cache-30.0.2.tar.gz.drv'...
building '/nix/store/bm473qznmr2nc6anwcaj7ysdfbd94yyv-crate-wat-1.227.1.tar.gz.drv'...
building '/nix/store/h25f8k8b8zzh36g19xzl7alc7j09j7r5-crate-wasmprinter-0.224.1.tar.gz.drv'...
building '/nix/store/nc3n06ykh22mw4jvfngp2317pzjrs16y-crate-wasmtime-math-30.0.2.tar.gz.drv'...
building '/nix/store/wrl2xh04p44iwrl92js7zj70hbwan718-crate-wasm-opt-0.116.1.tar.gz.drv'...
building '/nix/store/wk2j6zfqss74lj0af4ylyxl2hyv11s20-rustc-nightly-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/sdaii3f0ssp88rl9l1mx1a4as6zmwpf9-rust-std-nightly-wasm32-unknown-unknown.tar.xz.drv'...
building '/nix/store/izrps80529b52lyk8lg7q5ddax1998ix-rust-std-nightly-wasm32-wasip2.tar.xz.drv'...
building '/nix/store/qmgc3q768ach39zc49xsfxzpn9m40pq0-cargo-package-constant_time_eq-0.4.2.drv'...
building '/nix/store/1bi56q1vcmi0rby59qxch40yflpjfljg-cargo-package-icu_normalizer-2.1.1.drv'...
building '/nix/store/gsypdf1jl3vri5b4s47bhqdf4w886kyg-cargo-package-icu_provider-2.1.1.drv'...
building '/nix/store/4z4bi82a482rlwb2qm0idiyxd85q1942-cargo-package-toml_datetime-1.1.1+spec-1.1.0.drv'...
building '/nix/store/sqlzrwmng6383x39mpbrfr6s81a7lgjl-cargo-package-icu_properties_data-2.1.2.drv'...
building '/nix/store/6w0y27wlvgw3sg9ddjas1iwngmm2c3xy-cargo-package-idna-1.1.0.drv'...
building '/nix/store/f1x50w7f6v7pgj7wdmm6y6n8kss0l6v7-cargo-package-icu_normalizer_data-2.1.1.drv'...
building '/nix/store/mqlr7r2mv5zkb7mbh1dslfxf7z9rb0gk-cargo-package-crossbeam-deque-0.8.6.drv'...
building '/nix/store/dk0d50v4g4fbdcnbpj1qbiv321yv4px7-cargo-package-crossbeam-utils-0.8.21.drv'...
building '/nix/store/f7k035bbj4swg7hwfbm54f1lw8gim4k4-rustfmt-nightly-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/qz0bjd3caf01q38nzcd9jz86xnplim6s-cargo-package-blake3-1.8.4.drv'...
building '/nix/store/w8qgxq1qbki2h3mlxc340fp508zxyik6-cargo-package-arrayref-0.3.9.drv'...
building '/nix/store/my4lxvcjlc90ssyqyqxyshpydy4f9v3j-clippy-nightly-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/fj7hxn9fyqn36f43bggay2c88670q3kj-rust-src-nightly.tar.xz.drv'...
building '/nix/store/8mcykl0a4w8qnl9shfxwjdpfn9mghlcl-rustfmt-1.90.0-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/kji629zk8knf0x3g84dbfm8kxc4ffi3m-cargo-nightly-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/6r8k1k269avlygqrkgmsq4hrz3050b0m-rust-std-nightly-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/55xidslg6lpwslvqi4d95kfbhcfhk9r4-rustc-1.90.0-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/grkzirqls6figqx9xlk3kzlrrfl8dzf2-cargo-package-hashbrown-0.16.1.drv'...
building '/nix/store/j1diwgwnic00n43mdldimrbc7i3xvwfm-cargo-package-arrayvec-0.7.6.drv'...
building '/nix/store/5xc5yw2mnizcln8b4f52fmgxb861s4z7-check-store-retention-policy.drv'...
building '/nix/store/9b1vm01kxfxz18xv3sqh1iylk7rq3ch3-cargo-package-percent-encoding-2.3.2.drv'...
building '/nix/store/wnlgixpasmfr6kv7q1792jy8cmc3cxzr-cargo-package-memchr-2.7.6.drv'...
building '/nix/store/nx4fxa20hw991dlc02pz2fyq57aqz4xy-cargo-package-litemap-0.8.1.drv'...
building '/nix/store/13nwhbymbcf8yw5b6dwy2zj1qy8mkkak-cargo-package-idna_adapter-1.2.1.drv'...
building '/nix/store/adx5vknnsmbflzj0q3j9yiig5xc2f2yv-cargo-package-form_urlencoded-1.2.2.drv'...
building '/nix/store/mqmwqqm8xyklr9ishcvp7qzdq0m2skhj-cargo-package-icu_properties-2.1.2.drv'...
building '/nix/store/yi8rjpa59ahc8gwx624fb5ffjsk7k42b-cargo-package-libloading-0.9.0.drv'...
building '/nix/store/a8835mslzq6spxiyqk1rz9adpvlg3ldv-cargo-package-bitflags-2.11.0.drv'...
building '/nix/store/p6p59j7w06wn9hpwqphfx2wxlz1357al-cargo-package-cargo-platform-0.3.2.drv'...
building '/nix/store/4kgjln0h3wj0g6ngvm42zpd1vhzafjyh-cargo-package-git2-0.20.4.drv'...
building '/nix/store/byhain0wny40plg2045xpiwpzp7wa3fh-cargo-package-cc-1.2.57.drv'...
building '/nix/store/gw1jhqsrx02xmj3rlqdszghqb6xcll2q-cargo-package-cc-1.2.60.drv'...
building '/nix/store/9k3grr3fb3bil9k8z5fjyfqvw2wa805m-cargo-package-cargo_metadata-0.23.1.drv'...
building '/nix/store/3l1wcbqdr9fvvdlg10814pgv61792v28-cargo-package-fastrand-2.3.0.drv'...
building '/nix/store/nqclhj9vs4cvlnjh8isqjph0nxd1nnxh-cargo-package-pkg-config-0.3.32.drv'...
building '/nix/store/6f5bjpc266lxc5077syiyd1hixgkbkgk-cargo-package-regex-automata-0.4.13.drv'...
building '/nix/store/3axvi9lnq69m8allnndkarmv2ry30dv7-cargo-package-schannel-0.1.28.drv'...
building '/nix/store/05f64n521pr25l2zabnlyabh6f2hbx2r-cargo-package-semver-1.0.27.drv'...
building '/nix/store/bbc6s0kqwxsq9dqnralm8zgidal2mbl4-cargo-package-serde_spanned-1.1.1.drv'...
building '/nix/store/aj04dnskp1v6dmf6y9c7ifbwa511bd4q-cargo-package-smallvec-1.15.1.drv'...
building '/nix/store/m93jc9nr83rn3r7wj8iing3db5w7kanr-cargo-package-socket2-0.6.2.drv'...
building '/nix/store/f36y4gvm686sg1l4csc9w17kfl4krcx0-cargo-package-stable_deref_trait-1.2.1.drv'...
building '/nix/store/928ac284d2zwx93jwbaww2fa9644svvd-cargo-package-tinystr-0.8.2.drv'...
building '/nix/store/r83ni8bwx0qhlmh585nlg4pryc4dhhyn-cargo-package-toml_datetime-1.0.0+spec-1.1.0.drv'...
building '/nix/store/3m0c4bmcay70cdsrhc0jad5j0nmpka50-cargo-package-toml_edit-0.25.4+spec-1.1.0.drv'...
building '/nix/store/g9afbb16sqvf5wqx0y53x6sg0c0c49xf-cargo-package-url-2.5.8.drv'...
building '/nix/store/fnhfspa9r71nr3mqk1h4gc12pjldibvp-cargo-package-vcpkg-0.2.15.drv'...
building '/nix/store/6lqjs2aa3dvhc8wk5bg6pyc8bcgifppd-cargo-package-windows-sys-0.59.0.drv'...
building '/nix/store/bb20ksi8b9jyhw1xpmviigfms2rsnwnz-cargo-package-windows-targets-0.52.6.drv'...
building '/nix/store/2xsiamd7iggx22pza35g819ql30q05kn-cargo-package-windows_aarch64_gnullvm-0.52.6.drv'...
building '/nix/store/jndva7bcdmqh74sql6lfchfxj0mgbqid-cargo-package-windows_aarch64_msvc-0.52.6.drv'...
building '/nix/store/lakfyp9mgikbl28vbfjyqiymbbx49hmx-cargo-package-windows_i686_gnu-0.53.1.drv'...
building '/nix/store/zm4yghzgr6gvhqwld7a6dgamgmdx3i6c-cargo-package-windows_i686_gnullvm-0.52.6.drv'...
building '/nix/store/rlmidakq1b83kcdxna7ks5lnc3258cii-cargo-package-windows_x86_64_msvc-0.53.1.drv'...
building '/nix/store/iv4f8hzbxivc99lq3djk0kpdvjzmkacf-cargo-package-zerofrom-derive-0.1.6.drv'...
building '/nix/store/vy531grs0hmfls246rf8cyjz02zgafrm-dylint-driver-src.drv'...
building '/nix/store/x4w9iia9xrsm3qr5j54x7mxs0xpjfg82-Cargo.toml.drv'...
building '/nix/store/v9iw4gkc0w2yl6xir15mrxiqqhxpay2q-Cargo.toml.drv'...
building '/nix/store/qw25d64hp8lcphaqz7ggyp43v1p1xyy3-Cargo.toml.drv'...
building '/nix/store/qrdspgc29m93cqd2cz4xjg5xr625nmn5-Cargo.toml.drv'...
building '/nix/store/qkf4w37n993fkkhbk63fl5xsah73svg0-Cargo.toml.drv'...
building '/nix/store/ppla0v9k12sshi2157ydbxp4g51na7n2-Cargo.toml.drv'...
building '/nix/store/lgx38g1mvn0dv2qggznrhi40400wxxn6-Cargo.toml.drv'...
building '/nix/store/lbghk5vrqgm9pg9gdxifn88rq56blx63-Cargo.toml.drv'...
building '/nix/store/j4pyxlhwcnl50sl2yb00wy2cyfxwash5-Cargo.toml.drv'...
building '/nix/store/ic6dn9862z2dfj6xzcsl8alm0d8bgp24-Cargo.toml.drv'...
building '/nix/store/cviqkv4phl06iv6d9mwih8kv6ymaswzz-Cargo.toml.drv'...
building '/nix/store/cdl61wy3nq8p6mj5pr9i2g4mkxa59xfn-Cargo.toml.drv'...
building '/nix/store/6sv442x5sl0j2dj3z27wp88s1ks7ibg2-Cargo.toml.drv'...
building '/nix/store/0ksc756w8p7zgb089j83p4bxvypb17a9-crate-cranelift-codegen-shared-0.117.2.tar.gz.drv'...
building '/nix/store/wjpx007ddmrmc19bn5bbraklflbk9jfy-crate-cap-net-ext-3.4.2.tar.gz.drv'...
building '/nix/store/ll1f1b29rshw71fd8qpy7l1gklmg70jc-crate-cranelift-isle-0.117.2.tar.gz.drv'...
building '/nix/store/9laxv3i0h8iirnkszx3dcd744j3qbss4-directories-next-2.0.0.drv'...
building '/nix/store/klbrjby8azi2dz8j8dbh1pn2kb4iqxj0-crate-cranelift-bitset-0.117.2.tar.gz.drv'...
building '/nix/store/qr1sz2vbp7xhhjbm9nzk0b4z67za7pnd-Cargo.toml.drv'...
building '/nix/store/7v6yf6vhx12pwrrvz848wpj157a6b18v-Cargo.toml.drv'...
building '/nix/store/0mc552cja9gsfc2zp4hs5zjp3f8xsgmr-crate-cap-rand-3.4.2.tar.gz.drv'...
building '/nix/store/2i76ch3p5ww7a42c8hr4zhczhk9az5f7-crate-iana-time-zone-0.1.62.tar.gz.drv'...
building '/nix/store/7y1pq94rmcc2v2kvml4dnyzhamjzb7f1-crate-wasm-encoder-0.227.1.tar.gz.drv'...
building '/nix/store/009y8prddh7yfifni0a8pdn5db28p8jk-crate-wasm-encoder-0.217.1.tar.gz.drv'...
building '/nix/store/xc7lagvgjxr7gk7gv3b8qaii96z2ja27-crate-wasm-compose-0.219.2.tar.gz.drv'...
building '/nix/store/z4dghs5sv46ccl212brbnakv78gw0xkm-cap-rand-3.4.2.drv'...
building '/nix/store/ihp64py94w8jgywssg8w9aw257sw2q1c-crate-cxxbridge-flags-1.0.128.tar.gz.drv'...
building '/nix/store/y4v4kdb89qlc79i2zdqvr06n3hkzf9v5-iana-time-zone-0.1.62.drv'...
building '/nix/store/x20z57mfs3i3fnprw6bbdi4piwm2f3vw-wasm-encoder-0.217.1.drv'...
building '/nix/store/9shsmkf0yn5y1vp6dl2ixqada6qibxqs-wasm-encoder-0.227.1.drv'...
building '/nix/store/vy4dmyc4fd5gfn02npacafjpxyzqcjy0-crate-cranelift-codegen-0.117.2.tar.gz.drv'...
building '/nix/store/41maim1rfcha3jb4hf7six5inglqwzmd-crate-regalloc2-0.11.1.tar.gz.drv'...
building '/nix/store/33j13ylpf26ph1fbr16sy13m9gb7vxkz-crate-walrus-0.23.3.tar.gz.drv'...
building '/nix/store/2n9r7ny0py8qz532jvyl99vnimhx328y-wasm-compose-0.219.2.drv'...
building '/nix/store/7q9fl71h60pzdv49b64432ph0nn0hsfm-cxxbridge-flags-1.0.128.drv'...
building '/nix/store/q6hnq6k5irgyhzc7n9dmriy2p7j5ral4-crate-cranelift-native-0.117.2.tar.gz.drv'...
building '/nix/store/j7ll9ldcs7kg81wasy94ycz52bbyqckm-im-rc-15.1.0.drv'...
building '/nix/store/ssdvmg76nwhhwmkzm8wmcii7l1r7xml2-regalloc2-0.11.1.drv'...
building '/nix/store/nxwmz9dj2h2yx683np8wpj8ps0db43mp-cranelift-codegen-0.117.2.drv'...
building '/nix/store/lpqvf9jaqpnd7icj8559jg3cn4q6pgk6-walrus-0.23.3.drv'...
building '/nix/store/h9vas4hs8sf2d3ragfyrpmm9rb8swsfc-cargo-package-libc-0.2.185.drv'...
building '/nix/store/5ld2clmls20wwlavr051yl9wn6w78cca-crate-cranelift-bforest-0.117.2.tar.gz.drv'...
building '/nix/store/115ckilzcv0z1jghawb2qdlz1hlj6vk8-crate-wasmparser-0.227.1.tar.gz.drv'...
building '/nix/store/3sp9cm6xb5p4zibn4zz314f1n48y4pzg-crate-wasmparser-0.224.1.tar.gz.drv'...
building '/nix/store/fz3dz0aijxkqla744dxzhhxzrh1550ax-cranelift-native-0.117.2.drv'...
building '/nix/store/jvgscjmm5klvp3d8g3pammvkkykpwg2s-rust-std-1.90.0-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/vycqfd9m3mp739mlf1sah1vdkjl1ls2j-cranelift-bforest-0.117.2.drv'...
building '/nix/store/cfg2n276xqjx2xsnpc6m7qdgc5c8zwmw-cargo-src-winnow-1.0.1.drv'...
building '/nix/store/37f1c2xrylv8a3kzzc8hjs1w49j9n2jp-linkLockedDeps.drv'...
building '/nix/store/4k4j9lk6nwqr74qp779nlw3sak9hvxhg-rust-std-1.90.0-wasm32-wasip2.tar.xz.drv'...
building '/nix/store/adrxg9r16c8vh0hfnkpvb3ypy20yax7p-wasmparser-0.227.1.drv'...
building '/nix/store/7kf24lg8fpxkll8d3mm6ydp3ynykqq2g-wasmparser-0.224.1.drv'...
building '/nix/store/da4748p8i0bp3azzhnz72g7b4sm8v6mm-cargo-package-winnow-1.0.1.drv'...
building '/nix/store/6w2cj9rk1jkfbcrh703h5dddgjlg3zwq-vendor-cargo-deps.drv'...
building '/nix/store/3y940xy36zjg378xkgz0fp0dal8b561i-cargo-1.90.0-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/n6f52zwfyb5p6a53vf1axa1l650rayrz-clippy-1.90.0-x86_64-unknown-linux-gnu.tar.xz.drv'...
building '/nix/store/8rn7lzs56h934d6vcrhsx1wbq4qlsnrc-cargo-src-tokio-1.52.0.drv'...
building '/nix/store/7xxikklzyzzlrgkn0dd71kz0sjxh1ga0-crate-wasmparser-0.214.0.tar.gz.drv'...
building '/nix/store/ajd65n350311k1dxyavvdsjnkbcl87gf-cargo-package-tokio-1.52.0.drv'...
building '/nix/store/79azs8v3nggwcrinh6kwf72040nwcsjv-wasmparser-0.214.0.drv'...
building '/nix/store/az8gfpgxrd8xajikm5y58l6kbc8m2zzi-crate-wasmparser-0.217.1.tar.gz.drv'...
building '/nix/store/5dr6653i5nrbjkmj50hkksp8njm0dsrb-cargo-package-zerovec-derive-0.11.2.drv'...
building '/nix/store/83n6i77wf03zs0vk9f20j1mlhb9cxk22-cargo-1.90.0-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/b6qk91ryc160ny8haajw5lgl0nr3bzyy-wasmparser-0.217.1.drv'...
building '/nix/store/3p48n5f9ciz4psb9m6ddvb6s665pj6yc-cargo-package-zerotrie-0.2.3.drv'...
building '/nix/store/zaw3qc1bjy920qxj6zhhxrsz2icxyk6k-cargo-package-yoke-0.8.1.drv'...
building '/nix/store/1rkwb7yqsl7q02jckz42wfsr1wswk4pw-cargo-package-writeable-0.6.2.drv'...
building '/nix/store/sr2n2ziwhahhqxk9bmh3ih21i3gy6m1a-cargo-package-windows_x86_64_gnullvm-0.53.1.drv'...
building '/nix/store/9vkb9k8c16gf76mhhz8yxn19khn9y33g-cargo-package-windows_x86_64_gnullvm-0.52.6.drv'...
building '/nix/store/kl93486cnm0a2z6fibgadrs0dsvqnyw4-cargo-package-windows_x86_64_gnu-0.53.1.drv'...
building '/nix/store/378ic6k6rs8zg4gnd1fijzl8v9jhpzyf-cargo-package-windows_x86_64_gnu-0.52.6.drv'...
building '/nix/store/24ivk9h8pj5lzgfphk179y9hlbjkz5ln-cargo-package-windows_i686_msvc-0.53.1.drv'...
building '/nix/store/ii57mavjw79drfn6x55vzcyfpisz3kv3-cargo-package-windows_i686_msvc-0.52.6.drv'...
building '/nix/store/503lk0nvmd8wmz2lfipyhisgibkg5cq7-cargo-package-windows_i686_gnu-0.52.6.drv'...
building '/nix/store/jswxj2gk3nwxv11mv8hn9cif0dw3h5xi-rust-std-1.90.0-wasm32-wasip2.drv'...
building '/nix/store/903zqpl4a3jd1py6v9xjdfbxw86iin9p-cargo-package-windows_aarch64_gnullvm-0.53.1.drv'...
building '/nix/store/ca60ni0ylsl0kxchq0ci49ms5y6nmhgj-cargo-package-windows_aarch64_msvc-0.53.1.drv'...
building '/nix/store/bixwzglkb3nqq7gir405ch81w4xmhdcx-rust-std-1.90.0-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/mbz6wdp61wgmph3brkkafkns0fr7z6qg-cargo-package-windows-targets-0.53.5.drv'...
building '/nix/store/25pk0wpf8v0r5pv2ywjzch5bdpf748ix-cargo-package-windows-sys-0.60.2.drv'...
building '/nix/store/zm9zm88cwld38qslxnc42jnq93xrmz51-cargo-package-wasip2-1.0.2+wasi-0.2.9.drv'...
building '/nix/store/cb57h9s8a7yg8z8hf2b8jbkqfvr1qz3q-cargo-package-utf8_iter-1.0.4.drv'...
building '/nix/store/yw04z1gibjr2bjkbbvgs0iwv20mvy6qr-cargo-package-toml_writer-1.1.1+spec-1.1.0.drv'...
building '/nix/store/mpsdziwifis96amqjpabw73lnq6x9yb5-cargo-package-toml_writer-1.0.6+spec-1.1.0.drv'...
building '/nix/store/1fxmlddbsid7dafq20f5p72lf3fgyvyb-cargo-package-toml_parser-1.0.9+spec-1.1.0.drv'...
building '/nix/store/6praikkydhngb0fy4h3b47adbc9yfbxb-cargo-package-toml-1.1.2+spec-1.1.0.drv'...
building '/nix/store/5hqjgr3xln8k133vladpkndyzs3kwa4z-cargo-package-thiserror-impl-2.0.18.drv'...
building '/nix/store/vai8nq11x8iswsv01sf1pp4hf1bfrg21-cargo-package-toml-1.0.6+spec-1.1.0.drv'...
building '/nix/store/bx8b6g2s0lb3lz9cmijkfs5azr9bhxnx-cargo-package-thiserror-2.0.18.drv'...
building '/nix/store/x0lyv5fmya6n0la3cpmmrpwg75l8kc4s-cargo-package-serde_spanned-1.0.4.drv'...
building '/nix/store/acayvv9h442k1v778jj8lcbn3hlyb5w2-cargo-package-rustc_version-0.4.1.drv'...
building '/nix/store/2x5jh4v21n301rq9ivnxxm317a8d09n6-cargo-package-potential_utf-0.1.4.drv'...
building '/nix/store/w2441zshipdl8lamwibn63dkza891w6j-cargo-package-openssl-sys-0.9.111.drv'...
building '/nix/store/kxiykivk69y05jfg3ybv1j8j3lwqg1lm-cargo-package-openssl-probe-0.1.6.drv'...
building '/nix/store/j4r5y321iics0r4k4m8z6bv05fzcssg4-cargo-package-libz-sys-1.1.23.drv'...
building '/nix/store/c68hls3qs56cnm4phk2q94j2alznhjgj-cargo-package-libssh2-sys-0.3.1.drv'...
building '/nix/store/8ai6n22chjbwqrd5y69jqzc2lv8bxj8a-cargo-package-libc-0.2.183.drv'...
building '/nix/store/v76ny60ki2h39sii88lygv03q78zpgrh-cargo-package-jobserver-0.1.34.drv'...
building '/nix/store/wih6wh9bnmkdiqnn9xxfrw7gdg3cm7rr-cargo-package-icu_collections-2.1.1.drv'...
building '/nix/store/s0kxy7qiri17h1pm9jpbdib8n1f68y04-cargo-package-indexmap-2.13.0.drv'...
building '/nix/store/q6cixs7dfxl0h5dmi0hq5fg1cmxb31zd-cargo-package-displaydoc-0.2.5.drv'...
building '/nix/store/sfdkzxbj25y7q8g3rwl3qh22nypqygcv-cargo-package-getrandom-0.3.4.drv'...
building '/nix/store/dlsdnmkxqpcna2hxa67k5vnckichwwm4-cargo-package-crossbeam-epoch-0.9.18.drv'...
building '/nix/store/z95zk2k6fd21v8b2qwh01q1i1fcrpx08-cargo-package-curl-sys-0.4.85+curl-8.18.0.drv'...
building '/nix/store/dl2260rx69g5mbxkql891k87dfqki3qp-cargo-package-camino-1.2.2.drv'...
building '/nix/store/72jfypl7j7ipsznrpxhxzhd2n2la2g1z-cargo-package-crossbeam-channel-0.5.15.drv'...
building '/nix/store/hivw1q57gmkhcpimpik6cfzskvikzp69-Cargo.toml.drv'...
building '/nix/store/lwa60jfmiini59iihwzjw4i3zgbs3zxx-crate-cranelift-entity-0.117.2.tar.gz.drv'...
building '/nix/store/pjpa2ly3ia080mvqsvd9smymaia84ayh-crate-wasmtime-component-util-30.0.2.tar.gz.drv'...
building '/nix/store/fjk7g5vf39wg8knbw5gny7pylfiyyv66-crate-wasmtime-cranelift-30.0.2.tar.gz.drv'...
building '/nix/store/6r46k6caknjsqwbdwbm6k6c15d63wc6k-vendor-registry.drv'...
building '/nix/store/rqbrk7yxpljmx0pyi90ways4vxivwqn6-vendor-registry.drv'...
building '/nix/store/73s8g7n83sxrfa69hdnag6p8lh23la05-vendor-cargo-deps.drv'...
building '/nix/store/42r59i1afv263fc4r0n77jnrxxiqaclh-source.drv'...
building '/nix/store/iasbgpgh19sp42dy276ga1z576b5jriw-cranelift-entity-0.117.2.drv'...
building '/nix/store/v35pbl31rvdk4d8hn1c1qjn1ln25yx3z-crate-wasmtime-asm-macros-30.0.2.tar.gz.drv'...
building '/nix/store/zinj21kbvsqcqgqwh8g948vrc083qsbm-crate-wasmtime-environ-30.0.2.tar.gz.drv'...
building '/nix/store/lz6a8088nw8ghv82qkr0f6v5dbxqadwg-wasmtime-cranelift-30.0.2.drv'...
building '/nix/store/bhqvfy2awc120rdin9iriz4z0q0ddpdj-wasmtime-component-util-30.0.2.drv'...
building '/nix/store/pk0r00z2mf8vgfxis67pv7r7kfvxn9p4-crate-wast-227.0.1.tar.gz.drv'...
building '/nix/store/s72gl2kmxmv3rl2ckv143q5s2vqmisgd-crate-wasmparser-0.219.2.tar.gz.drv'...
building '/nix/store/kmdk1m26yv5ark1zkfhv0qkvbkv2x0kw-octet-deps-0.1.0.drv'...
building '/nix/store/lwfvkbjpxdhiad3l1wg4amfwn1sgpin6-wasmtime-asm-macros-30.0.2.drv'...
building '/nix/store/smjh5irc22r410i9vx7z2dxcw226x5n6-wasmtime-environ-30.0.2.drv'...
building '/nix/store/14bil3613xj3yrg0p04qxy28qpymhmzs-crate-walrus-macro-0.22.0.tar.gz.drv'...
building '/nix/store/h8qjng7f5knm9cdfagyqfj1lc27b83rn-wast-227.0.1.drv'...
building '/nix/store/qcrc126way0r749ilkyi5avla3dypnbk-crate-r-efi-5.1.0.tar.gz.drv'...
building '/nix/store/4jmzp753mkdng469xq59wwbw1pgjdny3-wasmparser-0.219.2.drv'...
building '/nix/store/50v1jj4cyzrbi29rin1j82glhh32f15y-crate-wit-parser-0.217.1.tar.gz.drv'...
building '/nix/store/f6jvsvx5lqclndh3f9gl586kg1i6nz2a-crate-wit-bindgen-core-0.32.0.tar.gz.drv'...
building '/nix/store/cdaw2y80d2d2ssq16mdchny7y6nchgz9-walrus-macro-0.22.0.drv'...
building '/nix/store/bgwli46rhhvhp5ykg94d1zb6bafbq93d-r-efi-5.1.0.drv'...
building '/nix/store/4rsrkq191fxg3si5s76qb8391s60gywk-crate-winch-codegen-30.0.2.tar.gz.drv'...
building '/nix/store/rimv7vv9268pj9mxfdn6cg05rpmpcfqd-crate-wit-parser-0.224.1.tar.gz.drv'...
building '/nix/store/7l54dlakbpzd21vi97d8lwvrkdgq9n0f-wit-bindgen-core-0.32.0.drv'...
building '/nix/store/kvngqg59dibiwr9ybhgzpz8fw4an7lx0-winch-codegen-30.0.2.drv'...
building '/nix/store/gzff8h3j1ls5ifsn2wja66km37zhvfsh-wit-parser-0.217.1.drv'...
building '/nix/store/6fh7silqa6sipkg6pm9kihx302qk4fy2-crate-cranelift-assembler-x64-meta-0.117.2.tar.gz.drv'...
building '/nix/store/d0c71p3czql4zf54ldkw135zxyvsfmmy-crate-cranelift-codegen-meta-0.117.2.tar.gz.drv'...
building '/nix/store/8f9vzjiw4r1a313sjmlbpl2a7jprd3fl-crate-wit-component-0.217.1.tar.gz.drv'...
building '/nix/store/nkdgqsmgvrhr7qy8q0xrfdwnxwcy8223-wit-parser-0.224.1.drv'...
building '/nix/store/l9mzhqfhzdf5ra9aa8ra3ay52d5rs780-cranelift-assembler-x64-meta-0.117.2.drv'...
building '/nix/store/ifi919n2sbz3v8jfc0f80nlvyv7wscbs-crate-wasm-encoder-0.219.2.tar.gz.drv'...
building '/nix/store/ik0vz26f12yk6pb1sik0m6vgybwg610l-cranelift-codegen-meta-0.117.2.drv'...
building '/nix/store/n91b8w6psylkdbhw7j1py5csjl75i1hb-wit-component-0.217.1.drv'...
building '/nix/store/pf5awz4618pr43fbvwhc0hcxlbjvq06s-crate-pulley-interpreter-30.0.2.tar.gz.drv'...
building '/nix/store/2q3s1cpvijyjsl2qrmhz0xx817v996bb-crate-spdx-0.10.8.tar.gz.drv'...
building '/nix/store/lk78cfa1zsnni6i2pz5wxhxih2hbvvhs-crate-wasmtime-component-macro-30.0.2.tar.gz.drv'...
building '/nix/store/2n0awdw3chy18hg44vb1f4fnf67ipv7v-wasm-encoder-0.219.2.drv'...
building '/nix/store/wkgn0nwgk3i7dji46ffs76q9p860wjp2-pulley-interpreter-30.0.2.drv'...
building '/nix/store/b8zzfzr8sd1bi6p80yqgflyvj08z0j7w-crate-wasm-encoder-0.224.1.tar.gz.drv'...
building '/nix/store/4nhngzmnlsn4qcr13avkj4kv2lbx195c-crate-cxx-build-1.0.128.tar.gz.drv'...
building '/nix/store/yr1x2y67glc87cif5vq9gk1wjwy9wc15-spdx-0.10.8.drv'...
building '/nix/store/6p81i9q7qq29nab6f2bycinpsiaby3jj-wasmtime-component-macro-30.0.2.drv'...
building '/nix/store/lwk9bhclka2h9rig3na8kgvds37r5bhl-crate-wasmtime-wasi-config-30.0.2.tar.gz.drv'...
building '/nix/store/c1y2vknbra164akrc0ad9bfqmxqg80s4-wasm-encoder-0.224.1.drv'...
building '/nix/store/kda49gr149p4pvl55pkc4cr7h8w1nbqx-crate-wasmtime-wasi-30.0.2.tar.gz.drv'...
building '/nix/store/xqsy8vh4b0bph0wsaj7jrz0p0lkqdg9h-crate-wasmtime-versioned-export-macros-30.0.2.tar.gz.drv'...
building '/nix/store/bc0554jaza3cj9y3r6g3szv41dv15nxx-wasmtime-wasi-config-30.0.2.drv'...
building '/nix/store/h45kpllcnka8spigd2xijysj0f5y3w6x-cxx-build-1.0.128.drv'...
building '/nix/store/4jqr4nkzzw7z8amw60f3yinfplh3gvc2-crate-wasmtime-fiber-30.0.2.tar.gz.drv'...
building '/nix/store/s9qlizbbwnjsy0y4d6fg7s1k8r0miy6v-crate-wit-bindgen-rust-macro-0.32.0.tar.gz.drv'...
building '/nix/store/nlr23pgxlcjd4h40c3shc4avzi48fmsf-wasmtime-versioned-export-macros-30.0.2.drv'...
building '/nix/store/2c3fqz7a1nx2frp89pvyzjl5nqfvx4m5-wasmtime-fiber-30.0.2.drv'...
building '/nix/store/zz6qyliqp49hdp665ivdjrncjyv8vs5s-crate-wit-bindgen-rust-0.32.0.tar.gz.drv'...
building '/nix/store/ikn4d8gxdaxjjb3z0ys0bjfwj013avm7-crate-wit-bindgen-rt-0.32.0.tar.gz.drv'...
building '/nix/store/47w78y37fmnryd67xyx2zj396qjs8jnc-wasmtime-wasi-30.0.2.drv'...
building '/nix/store/jl5m2s37fyxg9p5w4002461nlnfyldwx-wit-bindgen-rust-macro-0.32.0.drv'...
building '/nix/store/7c0mry2zblzn48c6f4x4pygqvq1nwyq4-crate-cxx-1.0.128.tar.gz.drv'...
building '/nix/store/nyknr8kj5crakcxvxfzwfyvhfblsm6w4-crate-wasm-encoder-0.214.0.tar.gz.drv'...
building '/nix/store/hw5k6bnsd0crgqzizdc06qq9yypfd0xh-wit-bindgen-rust-0.32.0.drv'...
building '/nix/store/bjw3g4vvy2di4kj9vayg4md4v1nz439p-wit-bindgen-rt-0.32.0.drv'...
building '/nix/store/bdbxg7vb4bap4sdn6m8y7vhjzvbn1lds-crate-wasmtime-30.0.2.tar.gz.drv'...
building '/nix/store/9kz3j65yymk8w2998757nq6rhb58qgy6-wasmtime-wasi-io-30.0.2.drv'...
building '/nix/store/al1b3agh1lmnxq7rvwp7rhg5w8zyxqc1-cxx-1.0.128.drv'...
building '/nix/store/37zf0y8d2qnfqzyvipij35ls5vvn31h2-wasm-encoder-0.214.0.drv'...
building '/nix/store/c722zkkphajvl4hsdlavp879b5iph2g2-cxxbridge-macro-1.0.128.drv'...
building '/nix/store/bsmwzkalsnzqp5m8x28jprllk1ibcrgd-crate-wasmtime-winch-30.0.2.tar.gz.drv'...
building '/nix/store/67qf4wsa7xv8sdfmqgypfc7r0am7m64v-crate-wasm-metadata-0.217.1.tar.gz.drv'...
building '/nix/store/4fmm9d2abz9dhhkv3ng5xah4prhlqh95-wasmtime-jit-icache-coherence-30.0.2.drv'...
building '/nix/store/dx944ynf2qp7m43bszskqshjqrcw9gd4-wasmtime-30.0.2.drv'...
building '/nix/store/001gv3ssjpbrakzrrx5m4rasl58fxx7n-wasmtime-slab-30.0.2.drv'...
building '/nix/store/fn1shw7a20difjp2d72zc5ra94m9gxka-cranelift-frontend-0.117.2.drv'...
building '/nix/store/lckaxckyaydk4h8x3di31z0zgm0vkqap-wasm-metadata-0.217.1.drv'...
building '/nix/store/h5102g1hqibpqlphbjsz1znh3bpjzhp7-cap-fs-ext-3.4.2.drv'...
building '/nix/store/k5pky30dvffq23a6i12pg3nj1604dryi-wasmtime-winch-30.0.2.drv'...
building '/nix/store/xv5l05b70qwlqyyjx7yjkirri8y44dpl-wit-bindgen-0.32.0.drv'...
building '/nix/store/nvlc7g8wa0cf8k535jxnmx3c4fhgjgq1-wasmtime-cache-30.0.2.drv'...
building '/nix/store/lc6aa7wvhhxy94y702am9jrmr6d5hs9h-rust-docs-1.96.0-nightly-2026-04-08-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/mjgs0fh1ir6g2v3nz93kw3hsfjk33a32-rustc-dev-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/harx9h1vwimydy1yrhymi67vd6d02lzm-cap-time-ext-3.4.2.drv'...
building '/nix/store/xkirc9mj8pqrq55hi11irgiz5p8dzs56-wat-1.227.1.drv'...
building '/nix/store/ky4l8g3d81jmld91xm8g8zhnr4pzm22y-clippy-preview-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/y92yykrplhrq3v933hk4n39lx1mxk4b9-wasmtime-wit-bindgen-30.0.2.drv'...
building '/nix/store/4cgmg0fnak3zi1xzi53m27zrbkhhs98n-wasmprinter-0.224.1.drv'...
building '/nix/store/vkhly7fzwf3dkiy5nibiv3m543lga82n-rustfmt-preview-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/iyds6r71ws91wp4q2xkl2fizpc79kwb7-rust-std-1.96.0-nightly-2026-04-08-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/9xjxb3nxshrqv98zdm9mf5jr84wmxzpr-wasm-opt-0.116.1.drv'...
building '/nix/store/wssm7alr4pzz99hgfggmljqkpmkyklj5-wasmtime-math-30.0.2.drv'...
building '/nix/store/yjhawsj87b5scg8fv9ss42sgcys98d2p-rustc-1.96.0-nightly-2026-04-08-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/n8wjpjdxih91wd9jdl61xqgnwxqmjv88-rust-std-1.96.0-nightly-2026-04-08-wasm32-wasip2.drv'...
building '/nix/store/jjr2j9ackc0h4m3f6a3znphacd2n7c5a-rustc-1.90.0-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/i484hyy8k46qnnzqm8v29qf55lji8b3l-clippy-preview-1.96.0-nightly-2026-04-08-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/mdfdikl41mq9hh882bvkgq7lrbf94jgx-rustfmt-preview-1.96.0-nightly-2026-04-08-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/r828qh6m5chvka2d7fl2zv1hz4s8gwpn-octet-0.1.0.drv'...
building '/nix/store/p0gf4vkd5wk4wkh4dgrf3brwx94q1n4v-rust-default-1.96.0-nightly-2026-03-21.drv'...
building '/nix/store/hgdjy1ppapy162vxbvvakdic5vfyh631-cargo-octet-0.1.0.drv'...
building '/nix/store/pbka871dvffi8d33wvmgbkq8cpa7kc1r-cap-net-ext-3.4.2.drv'...
building '/nix/store/pcl0zln0gkry0sp1xwrnrh32bqqkm7l6-cargo.drv'...
building '/nix/store/lplifaf32na1qx6yj436b68qhbbwx85r-cargo-git-https-github.com-trailofbits-dylint-0e0a71eefe6f01563d11acc6e1d7af1d505934a9.drv'...
building '/nix/store/wp51zb243ajmicrb7hhfqzy5rdv7250f-octet-ui-library.drv'...
building '/nix/store/yj1w3j0gy338rkjng224ksphnh6vq0d1-clippy-preview-1.90.0-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/2m307icqp7865ajpckk50ir1f7q3a3n9-dylint-driver-5.0.0.drv'...
building '/nix/store/7krsjf39jxkmwb2m95j71izjxjyqmx47-rustfmt-preview-1.90.0-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/j55nsjq48y7155ndjpgxk2716723hlcg-rust-src-1.96.0-nightly-2026-04-08-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/v3a8isnzbz06v6rqyh6y13w2lfsfqgfp-rust-default-1.90.0.drv'...
building '/nix/store/cxqwb5p4wcyqxqcr8n3ij7jpwsy21kcg-linkLockedDeps.drv'...
building '/nix/store/7fnw09zlv4s4r3lx6c5szgm7xmwsxldw-rust-std-1.96.0-nightly-2026-04-08-wasm32-unknown-unknown.drv'...
building '/nix/store/ad4ckb3sy1jjm8rpaj9n1cf6s3llanyr-vendor-cargo-deps.drv'...
building '/nix/store/xihq3v8nv2n30w3pcny8q3ldklf5800x-cargo-1.96.0-nightly-2026-04-08-x86_64-unknown-linux-gnu.drv'...
building '/nix/store/1lwbc1kb2jaf6z37qppggjm52rfk4fyp-cranelift-codegen-shared-0.117.2.drv'...
building '/nix/store/b54vpj8zhx66k10f81bk1ypqjvbc18r7-cranelift-isle-0.117.2.drv'...
building '/nix/store/kcddhgd4w00cvf50jyqnjhhayci0khhy-source.drv'...
building '/nix/store/p1bvgyrlghghi6i0w38lrpd34wvrm9w7-cranelift-bitset-0.117.2.drv'...
building '/nix/store/j87qrwfzafc7v7j21z1rb12wc76iyiy1-tigerstyle-deps-0.1.0.drv'...
building '/nix/store/qr8w0ndwv0qqihcsicw6gn7p0cx5jsqr-rust-default-1.96.0-nightly-2026-04-08.drv'...
building '/nix/store/m924s5km4vnl5sr184yswzkvfcwqfpzr-terminal_size-0.4.2.drv'...
building '/nix/store/5nyixg0n3c7ja7c6fw5pnyza9qiad601-serde_json-1.0.128.drv'...
building '/nix/store/lz5fjdv6w21m3534kfhxhl8wphp6mqgr-addr2line-0.24.1.drv'...
building '/nix/store/8i1hgpav0bw2f597iq3l0yk1m6b5d5i5-cc-1.2.17.drv'...
building '/nix/store/k6vp51194rnfymczwkzhfqpq5b7sif8g-cobs-0.2.3.drv'...
building '/nix/store/bx2vfl4jfnyq89v4wma1qhbxxpyzvzkj-id-arena-2.2.1.drv'...
building '/nix/store/8hwsj2hfskmbz76c9gk2f4p0ffrcj711-gimli-0.31.0.drv'...
building '/nix/store/g9pwvkn5ijs75c7gxb56z3kgipsr4lf9-mach2-0.4.2.drv'...
building '/nix/store/k0nv062g5785xsp8nkjjxf5msq5bqhi0-memfd-0.6.4.drv'...
building '/nix/store/na4hv3n49z7kkv2far7da4jy5h9yvnc2-prettyplease-0.2.25.drv'...
building '/nix/store/x91z17w20zpd4qm3hia6753f6ai024zl-sptr-0.3.2.drv'...
building '/nix/store/qrb8kbciz5vxvx63zrysh161zwy1km5a-linux-raw-sys-0.9.3.drv'...
building '/nix/store/h25snc3gv50h9m81lgwmcvyhrianj9s5-link-cplusplus-1.0.9.drv'...
building '/nix/store/d8p70i34ljnjdg9vl79vnxidh43406iy-clap_derive-4.5.47.drv'...
building '/nix/store/b31w03nqbfyp3gcyc17pjglffzakr23p-gimli-0.26.2.drv'...
building '/nix/store/fdgk7j2vcg0br5xim3y7f5cxhx7117bq-postcard-1.1.1.drv'...
building '/nix/store/rbalmfn56qfybvgw67dz2ql4kcm84b87-psm-0.1.25.drv'...
building '/nix/store/36jlsgl3ncddgfibfsr9ksmdjkhgjnjh-rustix-1.0.3.drv'...
building '/nix/store/1ag0maqkrw4j6szph4bjn220d7329x4i-scratch-1.0.7.drv'...
building '/nix/store/vrninn590zw2w6q45wgypxsihk12y5gz-tempfile-3.12.0.drv'...
building '/nix/store/a8jfcijbwqbrfa93d5i90y6hcvj392n4-wasm-opt-cxx-sys-0.116.0.drv'...
building '/nix/store/qd225cx6l39spymr61xbnn3nzhsvfdzx-winnow-0.7.11.drv'...
building '/nix/store/h6hx7n9f4wl2qjzmlr0glvw7bi9ar250-crate-cranelift-assembler-x64-0.117.2.tar.gz.drv'...
building '/nix/store/mbcl27kbmbacjscwias9s56k1ppgby37-crate-cranelift-control-0.117.2.tar.gz.drv'...
building '/nix/store/ah3gciabqrb46i4swaq8qc29fc55cv5p-cranelift-assembler-x64-0.117.2.drv'...
building '/nix/store/7g3x7dk5s4j1mi4zqwibfsgb8vn7mjvw-cranelift-control-0.117.2.drv'...
building '/nix/store/nvfjxw7vasrawl0zllx0cpm0pnyi9zsb-cargo-vendor-dir.drv'...
building '/nix/store/hx25vagbjn1i2615wqx7q52wqj4bcbhx-wasi-virt-0.2.0.drv'...
building '/nix/store/0jndlaqmrv4pkycyknbpvj405kgz6s1q-cargo-tigerstyle-0.1.0.drv'...
building '/nix/store/if6i0vs9h31k6ka20myaznpdh09adcbi-tigerstyle-0.1.0.drv'...
building '/nix/store/1h0kddkn33p4dyrkjnbzsp3ihl49gk77-tigerstyle-ui-library.drv'...
building '/nix/store/d8smv7g4fiv26dr9zha2yg9z2g4n5qzp-cargo-tigerstyle-0.1.0.drv'...
building '/nix/store/qz9jg163892lfkixcgxljbnma45nq2ry-cargo-octet-0.1.0.drv'...
building '/nix/store/jip7bl880iii0mzaywr3jzfdmfpkb1gb-mantle-wasm-component-toolchain-v1.drv'...
building '/nix/store/8w5qn10qlr05rzs2k7jy6wkk9lbnvg0w-nix-shell-env.drv'...
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
 Downloading crates ...
  Downloaded rstest v0.26.1
  Downloaded tokio-retry v0.3.0
  Downloaded rstest_reuse v0.7.0
  Downloaded hex-literal v0.4.1
  Downloaded async-io v2.6.0
  Downloaded relative-path v1.9.3
  Downloaded async-signal v0.2.13
  Downloaded async-process v2.5.0
  Downloaded rstest_macros v0.26.1
    Blocking waiting for file lock on artifact directory
   Compiling rustix v1.1.4
   Compiling regex-automata v0.4.14
   Compiling serde_json v1.0.149
   Compiling reqwest v0.13.2
   Compiling rand_chacha v0.3.1
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-castore)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/nix-compat)
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/fuse-backend-rs)
   Compiling radix_trie v0.2.1
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/nix-compat-derive)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-store)
   Compiling tokio-retry v0.3.0
   Compiling rand v0.8.6
   Compiling rstest_reuse v0.7.0
   Compiling regex v1.12.3
   Compiling reqwest-middleware v0.5.1
   Compiling object_store v0.14.0
   Compiling rstest_macros v0.26.1
   Compiling reqwest-tracing v0.6.0
   Compiling polling v3.11.0
   Compiling xattr v1.6.1
   Compiling tempfile v3.27.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-tracing)
   Compiling astral-tokio-tar v0.6.3
   Compiling async-io v2.6.0
   Compiling rstest v0.26.1
   Compiling async-signal v0.2.13
   Compiling async-process v2.5.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.48s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/snix_store-f71e76bb0a2d5287)

running 33 tests
test proto::tests::pathinfo::convert_without_narinfo_fail ... ok
test proto::tests::pathinfo::convert_inconsistent_narinfo_reference_name_digest ... ok
test proto::tests::pathinfo::convert_invalid_narinfo_reference_name ... ok
test proto::tests::pathinfo::convert_wrong_nar_sha256 ... ok
test nar::import::test::ingest_with_cahash_correct::case_3_flat_md5 ... ok
test nar::import::test::single_symlink ... ok
test nar::import::test::ingest_with_flat_non_file::case_2_nar_symlink_sha1 ... ok
test nar::import::test::ingest_with_cahash_correct::case_4_nar_symlink_sha1 ... ok
test nar::import::test::complicated ... ok
test nar::import::test::ingest_with_cahash_mismatch::case_1_nar_sha256 ... ok
test nar::import::test::ingest_with_flat_non_file::case_1_nar_sha256 ... ok
test nar::import::test::single_file ... ok
test nar::import::test::ingest_with_cahash_mismatch::case_4_nar_symlink_sha1 ... ok
test nar::import::test::ingest_with_cahash_correct::case_2_nar_sha512 ... ok
test nar::import::test::ingest_with_cahash_mismatch::case_3_flat_md5 ... ok
test nar::import::test::ingest_with_cahash_mismatch::case_2_nar_sha512 ... ok
test nar::import::test::ingest_with_cahash_correct::case_1_nar_sha256 ... ok
test tests::nar_renderer_seekable::read_to_end::case_2_helloworld ... ok
test tests::nar_renderer::single_file_missing_blob ... ok
test tests::nar_renderer_seekable::read_to_end::case_4_too_small ... ok
test tests::nar_renderer::seekable::case_1_symlink ... ok
test tests::nar_renderer_seekable::seek_twice ... ok
test tests::nar_renderer::seekable::case_3_too_big ... ok
test tests::nar_renderer_seekable::read_to_end::case_5_complicated ... ok
test tests::nar_renderer_seekable::read_to_end::case_1_symlink ... ok
test tests::nar_renderer::seekable::case_5_complicated ... ok
test tests::nar_renderer::seekable::case_2_helloworld ... ok
test tests::nar_renderer_seekable::read_to_end::case_3_too_big ... ok
test tests::nar_renderer_seekable::seek ... ok
test tests::nar_renderer::seekable::case_4_too_small ... ok
test pathinfoservice::nix_http::tests::narinfo_parsing_uses_the_configured_store_directory ... ok
test pathinfoservice::nix_http::tests::get_references_fetches_only_narinfo_metadata ... ok
test pathinfoservice::nix_http::tests::get_still_fetches_nar_payload_for_full_pathinfo ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.06s


exit_status: 0
```

### crunch-store baseline retry

```text
$ env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
 Downloading crates ...
  Downloaded bzip2 v0.6.1
  Downloaded libbz2-rs-sys v0.2.3
   Compiling tracing-subscriber v0.3.23
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-castore)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-build)
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-attestation-core)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/nix-compat)
   Compiling crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-gc-core)
   Compiling crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-repair-core)
   Compiling crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-overlay-core)
   Compiling crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-action-result-core)
   Compiling crunch-delta-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-delta-core)
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-attestation)
   Compiling tracing-indicatif v0.3.14
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-tracing)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-store)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-store)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-delta)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.09s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_store-fadb6f5ee5504d24)

running 336 tests
test action_result::tests::offline_discovery_never_opens_remote_sources ... ok
test action_result::tests::aggregate_candidate_limit_rejects_the_offending_source_response ... ok
test action_result::tests::duplicate_detached_signatures_are_rejected_before_publication ... ok
test archive::tests::import_action_core_rejects_untrusted_and_skips_present ... ok
test archive::tests::pathinfo_fixture_service_is_bounded ... ok
test action_result::tests::source_limit_bounds_empty_or_failing_remote_sources ... ok
test action_result::tests::local_interrupted_publication_is_not_discoverable ... ok
test archive::tests::archive_list_rejects_bad_magic ... ok
test archive::tests::archive_list_rejects_header_end_count_mismatch ... ok
test action_result::tests::local_conflicting_existing_record_is_not_overwritten ... ok
test action_result::tests::local_dangling_or_poisoned_index_fails_closed ... ok
test attestation::tests::artifact_attestation_file_uses_canonical_bytes ... ok
test build_io::tests::equal_length_replacement_preserves_binary_shape_and_occurrences ... ok
test attestation::tests::artifact_attestation_round_trips_by_logical_store_path ... ok
test action_result::tests::gc_retains_metadata_only_while_outputs_are_independently_live ... ok
test action_result::tests::local_publish_is_atomic_no_clobber_and_duplicate_safe ... ok
test build_io::tests::missing_blob_hash_fails_closed ... ok
test build_io::tests::read_file_node_rejects_non_file_and_over_limit_inputs ... ok
test build_io::tests::unequal_length_replacement_is_rejected - should panic ... ok
test ca_mapping::tests::insert_and_get ... ok
test ca_mapping::tests::load_missing_file_returns_empty ... ok
test capability::tests::missing_output_does_not_create_a_root ... ok
test ca_mapping::tests::save_and_load_roundtrip ... ok
test action_result::tests::discovery_only_base_is_readable_but_never_receives_publication ... ok
test closure::tests::cycle_terminates ... ok
test closure::tests::diamond_dedup ... ok
test closure::tests::linear_chain ... ok
test capability::tests::output_lookup_and_selected_root_registration_share_exact_identity ... ok
test closure::tests::local_query_error_can_still_use_remote_refs ... ok
test closure::tests::practical_mode_records_degraded_child_lookup ... ok
test attestation::tests::runtime_closure_attestation_uses_stored_artifact_digests ... ok
test closure::tests::practical_mode_records_degraded_root_lookup ... ok
test attestation::tests::runtime_closure_attestation_synthesizes_missing_member_artifact ... ok
test closure::tests::remote_fallback ... ok
test closure::tests::self_reference ... ok
test closure::tests::remote_metadata_lookup_failure_degrades_practical_mode ... ok
test attestation::tests::runtime_closure_attestation_is_cached_by_root_selection ... ok
test closure::tests::strict_mode_rejects_missing_root_closure_facts ... ok
test closure::tests::single_path_no_refs ... ok
test completeness::tests::blob_node_rejects_declared_size_mismatch ... ok
test completeness::tests::blob_node_requires_blob_presence ... ok
test completeness::tests::completeness_rechecks_and_rejects_removed_directory ... ok
test completeness::tests::chunked_blob_metadata_must_match_declared_size ... ok
test attestation::tests::runtime_closure_attestation_refreshes_after_member_digest_changes ... ok
test completeness::tests::symlink_is_always_complete ... ok
test completeness::tests::directory_with_missing_blob_child_is_incomplete ... ok
test completeness::tests::node_visit_limit_accepts_last_supported_node_and_rejects_overflow ... ok
test completeness::tests::empty_directory_requires_existence ... ok
test completeness::tests::bounded_depth_rejects_extremely_deep_trees ... ok
test archive::tests::archive_export_rejects_stale_final_nar_facts_before_writing ... ok
test archive::tests::ca_path_identity_accepts_reference_aware_standard_path ... ok
test archive::tests::missing_closure_reference_fails_export_plan ... ok
test archive::tests::archive_export_refuses_unsigned_without_escape_hatch ... ok
test closure::tests::depth_limit_enforced ... ok
test archive::tests::archive_export_list_round_trip_preserves_metadata_before_payload ... ok
test archive::tests::export_closure_includes_references_deterministically ... ok
test archive::tests::archive_list_drains_non_seekable_payloads_in_bounded_chunks ... ok
test build_io::tests::blob_and_nar_hashes_cover_all_supported_algorithms ... ok
test archive::tests::archive_import_rejects_store_prefix_mismatch_before_persisting ... ok
test archive::tests::archive_import_rejects_unsupported_ca_metadata_without_persisting ... ok
test archive::tests::archive_import_rejects_ca_metadata_for_another_store_path ... ok
test build_io::tests::rewrite_and_nar_hash_preserve_named_operation_behavior ... ok
test archive::tests::archive_import_rejects_conflicting_local_pathinfo ... ok
test archive::tests::archive_import_rejects_existing_path_with_stale_final_nar_facts ... ok
test archive::tests::archive_import_rejects_untrusted_signature_without_persisting ... ok
test gc::tests::file_cleanup_reports_failure_after_attempting_later_independent_paths ... ok
test action_result::tests::http_urls_strip_query_fragment_and_preserve_cache_subpath ... ok
test archive::tests::archive_round_trip_preserves_distinct_marker_ca_and_final_nar_identities ... ok
test archive::tests::archive_import_rejects_truncated_payload_without_persisting ... ok
test export::tests::export_directory_creates_files ... ok
test archive::tests::archive_import_rejects_tampered_payload_without_persisting ... ok
test gc::tests::path_explanation_rejects_retaining_root_links_above_policy_limit ... ok
test gc::tests::reclaim_observation_preserves_unknown_bytes_and_plan_identity_binds_shape ... ok
test gc::tests::remove_path_accepts_an_already_missing_export ... ok
test export::tests::export_directory_sets_permissions_and_mtime ... ok
test gc::tests::remove_path_does_not_follow_a_symlink_outside_the_export_tree ... ok
test gc::tests::remove_path_removes_nested_read_only_export_tree ... ok
test export::tests::export_directory_with_symlink ... ok
test gc::tests::snapshot_pathinfos_finishes_before_later_mutation ... ok
test export::tests::export_file_executable_sets_permissions ... ok
test export::tests::export_file_empty_content ... ok
test export::tests::export_file_rejects_short_blob ... ok
test export::tests::export_file_sets_mtime ... ok
test export::tests::export_file_non_executable_sets_permissions ... ok
test export::tests::export_file_writes_content ... ok
test export::tests::export_missing_blob_returns_error ... ok
test gc::tests::operation_reporting_preserves_first_failure_and_records_later_work ... ok
test handle::tests::action_result_nar_byte_accounting_distinguishes_transfer_reuse_and_overflow ... ok
test gc::tests::gc_aborts_when_root_registry_is_corrupt ... ok
test archive::tests::archive_import_round_trip_and_skip_existing_are_idempotent ... ok
test export::tests::export_missing_directory_returns_error ... ok
test handle::tests::delta_capability_url_preserves_cache_subpath_with_trailing_slash ... ok
test handle::tests::delta_capability_url_preserves_cache_subpath_without_trailing_slash ... ok
test handle::tests::delta_capability_url_uses_root_cache_authority ... ok
test gc::tests::explicit_castore_root_survives_while_unreachable_blob_is_reclaimed ... ok
test handle::tests::local_protocol_v1_matches_crunch_delta_wire_contract ... ok
test gc::tests::directory_outputs_survive_reopen_and_gc ... ok
test export::tests::export_file_creates_parent_directories ... ok
test export::tests::export_symlink_creates_link ... ok
test export::tests::export_nested_directory ... ok
test gc::tests::gc_aborts_when_retained_root_pathinfo_is_missing ... ok
test export::tests::export_moderate_depth_succeeds ... ok
test export::tests::export_symlink_sets_lmtime ... ok
test gc::tests::pathinfo_rewrite_failure_stops_before_export_deletion ... ok
test action_result::tests::http_interrupted_index_publication_leaves_record_undiscoverable ... ok
test gc::tests::gc_operation_order_matches_design ... ok
test handle::tests::overlay_missing_base_fails_closed ... ok
test handle::tests::overlay_duplicate_base_declaration_fails_before_overlay_creation ... ok
test gc::tests::dry_run_reports_same_candidates_as_real_run ... ok
test gc::tests::stale_plan_is_rejected_after_export_symlink_substitution ... ok
test gc::tests::stale_plan_is_rejected_after_root_change_without_deletion ... ok
test gc::tests::shared_blob_survives_when_reachable_path_still_references_it ... ok
test gc::tests::retained_root_keeps_transitive_closure_and_sidecars ... ok
test action_result::tests::http_corrupt_record_and_poisoned_index_fail_closed ... ok
test handle::tests::cached_node_for_path_reuses_local_pathinfo_node ... ok
test handle::tests::cached_node_for_path_rejects_incomplete_session_node ... ok
test handle::tests::action_result_admission_preserves_current_detailed_artifact_attestation ... ok
test handle::tests::overlay_base_state_mutation_blocks_output_admission ... ok
test handle::tests::failed_persist_does_not_register_root ... ok
test handle::tests::overlay_prefix_mismatch_fails_closed ... ok
test handle::tests::check_cache_accepts_ca_mapping_with_custom_store_prefix ... ok
test handle::tests::check_cache_directory_output_with_missing_child_is_castore_incomplete ... ok
test gc::tests::unreachable_output_removes_pathinfo_exports_and_attestations ... ok
test handle::tests::overlay_rejects_base_with_write_permission ... ok
test handle::tests::check_cache_replaces_stale_artifact_attestation_facts ... ok
test handle::tests::noop_publisher_is_default_and_skips_all_outputs ... ok
test handle::tests::check_cache_preserves_current_detailed_artifact_attestation ... ok
test build_io::tests::host_path_hash_is_deterministic_and_missing_paths_fail_closed ... ok
test handle::tests::overlay_generation_drift_blocks_later_read ... ok
test handle::tests::overlay_shadowed_path_does_not_inherit_base_trust ... ok
test handle::tests::overlay_incomplete_shadow_blocks_complete_base_fallback ... ok
test handle::tests::overlay_read_through_base_hit_does_not_mutate_overlay ... ok
test handle::tests::overlay_ca_mapping_precedence_and_publication_are_layer_bounded ... ok
test handle::tests::overlay_gc_execution_rejects_stale_base_generation ... ok
test action_result::tests::http_publication_is_record_first_discoverable_and_duplicate_safe ... ok
test handle::tests::overlay_rejects_base_pathinfo_with_invalid_layer_signature ... ok
test handle::tests::practical_pathinfo_open_fallback_records_audit_event ... ok
test handle::tests::persist_signed_output_rejects_store_path_mismatch ... ok
test handle::tests::persist_signed_output_writes_artifact_attestation ... ok
test handle::tests::persist_signed_output_rejects_unsigned_pathinfo ... ok
test handle::tests::overlay_writes_route_to_overlay_only ... ok
test handle::tests::resolve_same_authority_endpoint_rejects_cross_origin_urls ... ok
test handle::tests::remote_trusted_key_parser_accepts_indexed_keys_and_rejects_duplicate_indexes ... ok
test handle::tests::strict_pathinfo_open_fallback_is_rejected ... ok
test handle::tests::persistent_output_does_not_fail_on_publisher_error ... ok
test handle::tests::persistent_output_calls_configured_publisher ... ok
test handle::tests::persist_signed_output_registers_self_build_root ... ok
test handle::tests::overlay_gc_rejects_base_to_overlay_reference ... ok
test handle::tests::persist_signed_output_registers_build_root ... ok
test handle::tests::overlay_shadows_base_pathinfo ... ok
test handle::tests::remote_substitution_registers_bootstrap_root ... ok
test handle::tests::overlay_gc_retains_base_reachability_without_base_mutation ... ok
test http_closure::tests::conflicting_digest_path_identity_is_rejected ... ok
test http_closure::tests::depth_limit_fails_closed ... ok
test http_closure::tests::diamond_and_cycle_are_deduplicated ... ok
test http_closure::tests::duplicate_reference_is_rejected ... ok
test http_closure::tests::incomplete_plan_and_active_request_fail_closed ... ok
test http_closure::tests::invalid_limits_and_identity_are_rejected ... ok
test http_closure::tests::linear_plan_imports_root_last ... ok
test http_closure::tests::member_limit_fails_closed ... ok
test http_closure::tests::observation_without_active_request_is_rejected ... ok
test http_closure::tests::one_member_plan_is_stable_and_root_last ... ok
test http_closure::tests::reference_limit_fails_before_pending_members_change ... ok
test handle::tests::remote_substitution_without_cache_url_skips_probe_and_full_fetches ... ok
test http_closure::tests::reference_order_does_not_change_plan_identity ... ok
test http_closure::tests::returned_path_mismatch_is_rejected ... ok
test http_closure::tests::total_nar_size_limit_fails_closed ... ok
test layer::tests::layered_value_preserves_exact_index_through_map ... ok
test layer::tests::store_layer_booleans_are_disjoint ... ok
test layer::tests::store_layer_display_includes_exact_base_index ... ok
test layer::tests::zero_service_index_is_overlay ... ok
test handle::tests::remote_substitution_writes_artifact_attestation ... ok
test metadata_cache::tests::evict_expired_removes_only_expired_entries ... ok
test metadata_cache::tests::force_refresh_disables_get ... ok
test metadata_cache::tests::load_empty_cache_from_nonexistent_file ... ok
test metadata_cache::tests::load_corrupt_file_returns_empty ... ok
test metadata_cache::tests::put_and_get_roundtrip ... ok
test metadata_cache::tests::put_replaces_existing_entry ... ok
test metadata_cache::tests::remove_removes_existing_entry ... ok
test metadata_cache::tests::remove_returns_false_for_missing_key ... ok
test metadata_cache::tests::save_and_reload_persists_entries ... ok
test handle::tests::root_export_refreshes_stale_materialized_path ... ok
test mutation_lock::tests::try_acquire_rejects_second_mutator ... ok
test handle::tests::try_substitute_remote_records_metadata_cache_on_hit ... ok
test metadata_cache::tests::save_evicts_excess_entries ... ok
test overlay::tests::embedded_overlay_policy_is_typed_and_bounded ... ok
test path_identity::tests::marker_normalized_ca_path_is_accepted ... ok
test path_identity::tests::mismatched_ca_path_is_rejected ... ok
test provenance::tests::byte_reference_admission_requires_a_valid_store_path_digest ... ok
test overlay::tests::identity_record_rejects_prefix_drift ... ok
test overlay::tests::writable_base_is_rejected_before_generation_admission ... ok
test overlay::tests::generation_observation_rejects_symlink_members ... ok
test handle::tests::verified_local_output_adoption_rejects_a_missing_physical_path ... ok
test provenance::tests::classifier_covers_data_elf_script_and_unknown_executable ... ok
test provenance::tests::cpio_and_gzip_initrd_readers_classify_nested_executable_scripts ... ok
test provenance::tests::equivalent_observation_order_canonicalizes_deterministically ... ok
test provenance::tests::exact_reference_resolution_accepts_declared_target_and_rejects_foreign_unknown_and_escape ... ok
test provenance::tests::generic_compressed_streams_are_bounded_and_scanned_as_single_payloads ... ok
test provenance::tests::identity_shape_is_bounded_and_stable ... ok
test handle::tests::try_substitute_remote_negative_miss_prevents_repeat_probe ... ok
test provenance::tests::malformed_and_bounded_containers_fail_closed ... ok
test provenance::tests::named_limits_all_fail_closed ... ok
test provenance::tests::castore_scan_accepts_complete_signed_blob_without_host_fallback ... ok
test provenance::tests::castore_directory_scan_detects_symlink_loop_without_following_links ... ok
test provenance::tests::policy_accepts_bounded_sorted_profile_paths_and_rejects_bad_limits ... ok
test provenance::tests::preserved_identity_paths_are_targets_not_untranslated_foreign_references ... ok
test handle::tests::verified_local_output_adoption_ingests_signs_and_persists ... ok
test provenance::tests::castore_scan_rejects_untrusted_and_receipt_inconsistent_pathinfo_before_blob_reads ... ok
test provenance::tests::castore_scan_reports_missing_blob_before_claiming_complete_traversal ... ok
test provenance::tests::shebang_accepts_profile_and_rejects_relative_missing_and_non_utf8_targets ... ok
test provenance::tests::symlink_resolution_rejects_escape_missing_and_loop ... ok
test provenance::tests::tar_reader_finds_hidden_unclassified_executable_and_path_escape ... ok
test handle::tests::overlay_reads_file_and_directory_content_from_distinct_bases_without_backfill ... ok
test handle::tests::verified_local_output_adoption_rejects_changed_existing_content ... ok
test handle::tests::overlay_report_records_selected_base_descriptor ... ok
test handle::tests::verified_source_ingest_preserves_exact_path_and_reuses_matching_content ... ok
test handle::tests::verified_source_ingest_rejects_conflicting_existing_content ... ok
test provenance::tests::castore_scan_counts_duplicate_nodes_and_enforces_duplicate_limit ... ok
test export::tests::export_depth_limit_enforced ... ok
test handle::tests::try_substitute_remote_fallback_to_subsequent_url_when_primary_missing ... ok
test handle::tests::remote_substitution_cross_authority_capability_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_404_delta_probe_falls_back_to_full_fetch ... ok
test handle::tests::overlay_two_bases_stack_in_declaration_order ... ok
test handle::tests::remote_substitution_malformed_delta_capability_json_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_probe_error_falls_back_to_full_fetch ... ok
test provenance::tests::lexical_relative_resolution_never_returns_a_path_outside_root ... ok
test action_result::tests::http_timeout_rejects_source_without_fabricating_lookup ... ok
test handle::tests::remote_substitution_closure_scoped_delta_accepts_requested_output_and_reports_reuse ... ok
test handle::tests::remote_substitution_missing_local_backing_content_is_absent_from_receiver_manifest ... ok
test handle::tests::remote_substitution_malformed_stream_json_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_accepts_delta_chunk_stream_without_full_fetch ... ok
test handle::tests::remote_substitution_untrusted_delta_pathinfo_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_directory_delta_without_local_directory_closure_falls_back_to_full_fetch ... ok
test handle::tests::delta_and_full_substitution_record_same_attestation_and_root_metadata ... ok
test pull::tests::http_closure_duplicate_reference_fails_before_nar_download ... ok
test pull::tests::http_closure_path_mismatch_fails_before_nar_download ... ok
test pull::tests::http_closure_plan_validator_rejects_before_nar_download_or_store_mutation ... ok
test handle::tests::remote_substitution_receiver_manifest_stays_bounded_to_requested_output ... ok
test pull::tests::http_closure_total_nar_limit_fails_before_nar_download ... ok
test pull::tests::http_closure_untrusted_root_fails_before_nar_download ... ok
test pull::tests::http_closure_narinfo_limit_fails_before_nar_download ... ok
test pull::tests::http_pull_detects_store_path_mismatch_and_client_metadata ... ok
test pull::tests::http_closure_missing_dependency_fails_before_nar_download ... ok
test pull::tests::http_closure_dependency_content_failure_keeps_root_absent ... ok
test pull::tests::http_pull_accepts_unsigned_when_trust_unsigned ... ok
test pull::tests::http_pull_rejects_cache_base_url_userinfo ... ok
test pull::tests::http_pull_blocks_cross_scheme_redirects ... ok
test export::tests::export_file_allows_many_small_reads ... ok
test pull::tests::http_closure_changed_dependency_nar_keeps_closure_absent ... ok
test pull::tests::http_pull_accepts_unknown_key_signature_when_trust_unsigned ... ok
test pull::tests::http_pull_does_not_recurse_into_missing_references ... ok
test handle::tests::remote_substitution_probes_delta_capability_once_per_session ... ok
test pull::tests::http_pull_continues_after_narinfo_fetch_transport_failure ... ok
test pull::tests::http_pull_export_failure_after_persistence_is_fatal ... ok
test pull::tests::http_pull_maps_narinfo_http_403_to_missing_nar_count ... ok
test pull::tests::http_closure_pull_discovers_all_metadata_and_imports_root_last ... ok
test pull::tests::pull_accepts_unsigned_when_trust_unsigned ... ok
test pull::tests::http_closure_refetches_incomplete_local_dependency ... ok
test pull::tests::pull_nonexistent_source_returns_error ... ok
test pull::tests::http_pull_maps_narinfo_http_5xx_to_parse_error_count ... ok
test pull::tests::http_pull_maps_nar_http_failure_to_missing_nar_count ... ok
test pull::tests::http_pull_handles_compressed_xz_nar ... ok
test pull::tests::pull_detects_nar_hash_mismatch ... ok
test push::tests::deriver_normalization_rejects_an_empty_base_name ... ok
test pull::tests::pull_rejects_store_dir_mismatch ... ok
test pull::tests::pull_rejects_untrusted_signature ... ok
test handle::tests::remote_substitution_stream_failure_falls_back_through_real_http_cache ... ok
test pull::tests::http_pull_rejects_absolute_nar_url_without_download ... ok
test push::tests::push_custom_store_dir_narinfo_uses_correct_prefix ... ok
test push::tests::push_idempotent_skip ... ok
test push::tests::push_includes_unsigned_when_trusted ... ok
test pull::tests::pull_single_signed_path_round_trip ... ok
test pull::tests::pull_skips_missing_nar ... ok
test query::tests::store_sign_adds_signature ... ok
test pull::tests::http_pull_parses_references_with_local_store_prefix ... ok
test provenance::tests::payload_classification_is_deterministic_for_arbitrary_bytes ... ok
test query::tests::store_sign_all_skips_already_signed_entries ... ok
test query::tests::store_sign_replaces_same_key_signature ... ok
test query::tests::store_sign_appends_different_key_signature ... ok
test push::tests::push_normalizes_deriver_suffix_and_writes_parseable_narinfo ... ok
test push::tests::push_narinfo_references_match ... ok
test pull::tests::pull_skips_already_present ... ok
test pull::tests::http_pull_rejects_malformed_narinfo_text ... ok
test push::tests::push_preserves_existing_nix_cache_info ... ok
test push::tests::push_multiple_paths ... ok
test push::tests::push_single_signed_path ... ok
test query::tests::verify_signatures_reports_untrusted_signer ... ok
test pull::tests::http_pull_store_dir_mismatch_is_hard_error_before_narinfo_fetch ... ok
test push::tests::push_skips_unsigned_by_default ... ok
test pull::tests::http_pull_rejects_narinfo_store_path_prefix_mismatch_after_matching_preflight ... ok
test query::tests::store_sign_then_verify_roundtrips_under_custom_prefix ... ok
test repair::tests::pure_plan_distinguishes_current_and_stale_facts ... ok
test repair::tests::pure_plan_rejects_each_unsafe_candidate ... ok
test query::tests::store_verify_signatures_rejects_wrong_prefix ... ok
test pull::tests::http_pull_rejects_untrusted_signature ... ok
test retention::tests::embedded_policy_is_typed_and_bounded ... ok
test query::tests::verify_signatures_accepts_trusted_key ... ok
test roots::tests::corrupt_registry_is_rejected_without_replacement ... ok
test roots::tests::legacy_record_migrates_to_protected_unmanaged_state ... ok
test roots::tests::managed_generation_batch_is_atomic_and_shares_generation_number ... ok
test roots::tests::project_generation_reuses_identity_and_advances_new_identity ... ok
test roots::tests::pin_rejects_nonexistent_and_unreadable_paths ... ok
test roots::tests::shell_lease_renewal_is_bounded_and_requires_lease_facts ... ok
test roots::tests::unmanaged_remote_registration_uses_remote_owner_scope ... ok
test roots::tests::register_root_survives_reload_with_versioned_provenance ... ok
test roots::tests::unpin_removes_existing_versioned_record ... ok
test pull::tests::pull_multiple_paths ... ok
test pull::tests::http_pull_nix_cache_info_redirect_rejection_warns_and_proceeds ... ok
test repair::tests::missing_exact_pathinfo_is_rejected ... ok
test repair::tests::incomplete_content_is_rejected_without_pathinfo_mutation ... ok
test repair::tests::dry_run_does_not_mutate_stale_pathinfo_or_attestation ... ok
test repair::tests::invalid_ca_identity_is_rejected_without_pathinfo_mutation ... ok
test repair::tests::current_pathinfo_is_an_idempotent_no_op ... ok
test repair::tests::unsigned_stale_pathinfo_is_rejected_without_mutation ... ok
test repair::tests::execute_repairs_facts_replaces_signatures_and_preserves_attestation_graph ... ok
test repair::tests::stale_artifact_attestation_is_rejected_without_pathinfo_mutation ... ok
test pull::tests::http_pull_skips_missing_narinfo_404 ... ok
test pull::tests::pull_with_path_filter ... ok
test pull::tests::http_pull_rejects_unsigned_when_trust_unsigned_is_false ... ok
test query::tests::store_verify_missing_for_path_not_on_disk_in_custom_store_dir ... ok
test repair::tests::pathinfo_persistence_failure_is_reported_without_mutation ... ok
test pull::tests::http_closure_diamond_fetches_shared_member_once ... ok
test pull::tests::http_closure_pull_reuses_complete_dependency ... ok
test pull::tests::http_pull_single_signed_path_round_trip ... ok
test pull::tests::http_pull_pathinfo_persistence_failure_is_fatal ... ok
test query::tests::store_verify_ok_for_exported_path_in_custom_store_dir ... ok
test query::tests::store_verify_mismatch_for_tampered_disk_content ... ok
test pull::tests::http_pull_network_failure_continues_for_remaining_paths ... ok
test pull::tests::http_pull_skips_already_present_without_narinfo_request ... ok
test pull::tests::http_pull_nix_cache_info_client_error_warning_proceeds ... ok
test pull::tests::http_pull_missing_and_malformed_nix_cache_info_both_proceed ... ok
test pull::tests::http_pull_maps_other_narinfo_http_statuses_to_parse_error_count ... ok
test pull::tests::http_pull_rejects_malformed_or_wrong_prefix_references_before_persistence ... ok
test pull::tests::http_pull_maps_other_nar_http_statuses_to_missing_nar_count ... ok
test pull::tests::http_pull_handles_gzip_bzip2_and_zstd_nar ... ok
test pull::tests::http_pull_keeps_path_prefixed_cache_urls_stable ... ok

test result: ok. 336 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s


exit_status: 0
```

### project refresh CLI baseline retry

```text
$ env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --test project_refresh_cli
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-attestation-core)
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/fuse-backend-rs)
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-tracing)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/nix-compat)
   Compiling artifact-auth-core v0.1.0 (ssh://git@github.com/OnixResearch/onix-artifact.git?rev=c932138d880ddf4c2967f4c024b489b5c0022bf1#c932138d)
   Compiling crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-overlay-core)
   Compiling crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-repair-core)
   Compiling crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-gc-core)
   Compiling crunch-rust-cache-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-rust-cache-core)
   Compiling crunch-eval v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-eval)
   Compiling crunch-wasm-component-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-wasm-component-core)
   Compiling crunch-shell-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-shell-core)
   Compiling chapter-tgz v0.1.0
   Compiling mantlepkgs-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/mantlepkgs-core)
   Compiling crunch-bootstrap-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-bootstrap-core)
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-release-core)
   Compiling crunch-hardware-simulation-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-hardware-simulation-core)
   Compiling crunch-kernelscript-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-kernelscript-core)
   Compiling crunch-shell v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-shell)
   Compiling crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-action-result-core)
   Compiling artifact-auth-ed25519 v0.1.0 (ssh://git@github.com/OnixResearch/onix-artifact.git?rev=c932138d880ddf4c2967f4c024b489b5c0022bf1#c932138d)
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-attestation)
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-project-core)
   Compiling crunch-hardware-simulation v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-hardware-simulation)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-castore)
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-glue)
   Compiling crunch-wasm-component v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-wasm-component)
   Compiling crunch-project v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-project)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-build)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-store)
   Compiling crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-rust-cache)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-build)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-delta)
   Compiling crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-rustc-wrapper)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 07s
     Running tests/project_refresh_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_refresh_cli-2457476b6180e750)

running 12 tests
test list_stale_reports_stale_and_failed_without_mutating_files ... ok
test refresh_partial_failure_writes_successes_and_exits_nonzero ... ok
test freshness_local_directory_probe_updates_lock_digest ... ok
test refresh_trusted_file_input_with_wrong_signature_rejects_lock_write ... ok
test freshness_no_network_mode_does_not_contact_http_probe ... ok
test freshness_http_json_template_refreshes_selected_stale_input ... ok
test build_fetch_policy_refresh_uses_expected_hash_without_network_resolution ... ok
test refresh_tarball_uses_unpacked_tree_hash_not_archive_bytes ... ok
test freshness_git_ref_probe_reports_stale_without_mutating_lock ... ok
test refresh_trusted_file_input_writes_lock_evidence_and_show_reports_claim ... ok
test refresh_git_input_locks_resolved_rev_and_tree_hash ... ok
test freshness_command_missing_timeout_and_output_limit_fail_deterministically ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s


exit_status: 0
```
