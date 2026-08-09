# Security policy

## Supported versions

Only the latest public release receives security fixes. Reports about the
development branch are welcome, but source snapshots are not supported release
artifacts.

| Version | Supported |
| --- | --- |
| `0.1.0` | Yes |
| `main` and other source snapshots | Development only |
| Any other release | No |

## Security contacts and private reporting

The `mcp-sync` maintainers are the project security contacts. Report a
suspected vulnerability through
[GitHub private vulnerability reporting](https://github.com/EnjoyableWork/mcp-sync/security/advisories/new).
This route is visible only to the reporter and repository security contacts.

Do not open a public issue, discussion, or pull request containing vulnerability
details, proof-of-concept material, credentials, tokens, private configuration,
or user data. If GitHub private reporting is unavailable to you, open a public
issue titled `Private security contact requested` with no technical details or
sensitive data; a maintainer will arrange a private follow-up route.

Include only what is needed to reproduce and assess the report:

- the affected `mcp-sync` version or commit;
- the operating system, architecture, command, and client involved;
- the security impact and prerequisites;
- minimal reproduction steps using synthetic, redacted values;
- any known workaround or suggested fix; and
- your preferred disclosure and credit details, if any.

Never submit a live secret. Revoke or rotate an exposed credential immediately,
then identify only its type and a redacted location in the private report.

## Response and coordinated disclosure

These are response targets rather than a service-level guarantee:

- acknowledge a new private report within three business days;
- provide an initial triage result within seven business days;
- provide a status update at least every fourteen days while remediation is
  active; and
- aim to remediate and coordinate disclosure within ninety days, adjusted for
  severity, exploitability, release safety, and reporter agreement.

The maintainers will validate the report, assess severity and affected versions,
prepare a fix or mitigation, and coordinate a disclosure date with the reporter.
Please keep details private until an agreed date or until the maintainers publish
a GitHub security advisory. Urgent active exploitation may require an earlier
release or disclosure. Credit is optional and will use only the name the reporter
approves for publication.

Reports about a third-party dependency should normally also go to that upstream
project. Use this private route as well when the dependency creates a concrete
`mcp-sync` impact or requires a project-side mitigation.
