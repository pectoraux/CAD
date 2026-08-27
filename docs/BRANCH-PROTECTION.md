# Repository Governance

The following GitHub settings are required for the architectural freeze to be enforceable in a hosted repository:

- protect `main`;
- require pull requests;
- require the `governance` status check;
- disallow direct pushes to `main` for the implementation agent;
- require Architect approval for implementation PRs;
- prohibit force-push on `main`;
- preserve the architecture-freeze commit/tag referenced by the release process.

Repository scripts are evidence and defense-in-depth; GitHub branch protection is the final hosted enforcement boundary.
