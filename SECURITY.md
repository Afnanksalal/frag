# Security Policy

Frag is an educational compiler and does not currently execute untrusted generated machine code.

Still, please report security issues responsibly, especially problems involving:

- unsafe filesystem writes through the CLI
- crashes on malformed input that could be used for denial of service
- generated Verilog that misrepresents checked source

## Reporting

Open a private security advisory on GitHub if available, or contact the maintainer through the GitHub profile:

https://github.com/afnanksalal

Please include:

- affected version or commit
- reproduction steps
- expected behavior
- actual behavior
- impact

## Supported Versions

Frag is pre-1.0. Security fixes target the main branch.
