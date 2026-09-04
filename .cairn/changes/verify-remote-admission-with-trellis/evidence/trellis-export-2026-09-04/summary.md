# Kamacite and Valence export

The first generator compile failed because a local binding shadowed the `assumptions` function. The exact error remains in `export.attempt1.log`.

The second run passed before the proof IR gained its complete requirement root. Its old identities remain in `export.attempt2.log`.

The final run used these immutable contracts:

- Kamacite revision `de710a092d351e829abfb288d46124e2db8e5b7f`.
- Kamacite profile `kamacite.trellis-proof-evidence-profile.v1`.
- Kamacite producer role `formal-proof-candidate`.
- Valence revision `27b8b2124e12b80718ded124274fec98bed7a581`.
- Valence schema `trellis.proof-evidence`.
- Valence role `property`.
- Valence outcome `accepted_formal_proof`.

The final artifact identities are:

- Canonical Kamacite bytes: 2,088.
- Canonical Kamacite BLAKE3: `85316de03248e110837ad4c88c77177aa6cea554c8e8cfbb31cec3c4615406f3`.
- Kamacite projection BLAKE3: `c94a6ec0251e9b01a9e5c07765f89422cb755c593003782f5abce361ba0e0707`.
- Valence artifact BLAKE3: `744fdec501178acf4513386801469f54b2f425aa7c5965709ce54f90b4e78ca4`.
- Valence logical receipt BLAKE3: `3a0ebbeb89fdb5ad91516b6fa4d69d758d4b8811905f61340b34e02925ed411f`.

The Valence report is valid and has no issues. The generator also rejected missing assumptions, wrong identity domains, wrong roles, and weakened non-claims.

The generator source and lock file in `generator/` are the exact files used for this export. They are evidence-only files.
