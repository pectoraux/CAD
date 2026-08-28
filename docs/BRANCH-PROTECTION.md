# Repository Governance

The following controls are required for the architectural freeze to be enforceable in a hosted repository:

- protect `main`;
- require pull requests as the sole delivery channel for implementation work, bound to the active Work Order by title;
- require the `baseline`, `governance`, and `pr-work-order` status checks, with branches up to date before merging;
- require an independent Architect review (governance process) for implementation PRs — the review authority is the Architect's verdict, not a GitHub approval count or a second GitHub identity;
- prohibit force-push on `main`;
- prohibit branch deletion on `main`;
- enforce all of the above on administrators;
- preserve the architecture-freeze commit/tag referenced by the release process.

Direct pushes to `main` by the implementation agent are prohibited by governance process and gated by the repository's own baseline/spec/work-order scripts. Repository scripts are evidence and defense-in-depth; GitHub branch protection (required checks, admin enforcement, no force-push, no deletion) is the final hosted enforcement boundary. Reviewer independence is an architectural invariant; a GitHub approval count is merely one possible enforcement mechanism and is not required.
