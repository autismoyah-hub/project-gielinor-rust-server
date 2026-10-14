# Security reporting

This is a development alpha restricted to loopback by its current executable. Do not deploy it as an internet-facing production service or reuse real game-account passwords.

Never submit passwords, private RSA exponents, tokens, player databases, private logs or cache archives in public issues. Report reproduction steps with synthetic accounts and sanitized traces. If GitHub private vulnerability reporting is available, use Security → Report a vulnerability; otherwise ask the owner to arrange a private channel before sharing exploit details. No unpublished email address or guaranteed response SLA is asserted.

The owner must review proposed authentication, persistence, transaction and network changes. Source packaging secret-pattern checks do not replace a security assessment. Build outputs, private inputs and `work/state` belong outside commits. A historical secret would require rotation and a separately approved repository/history remediation; deleting only the latest copy is insufficient.
