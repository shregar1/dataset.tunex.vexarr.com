# Security

Report security issues privately to the repository maintainers. Do not open a
public issue for vulnerabilities.

Security-sensitive areas:

- decompression input handling
- memory mapping
- path handling
- plugin metadata
- future cache and manifest loading

The CI security workflow runs `cargo audit` and `cargo deny check`.
