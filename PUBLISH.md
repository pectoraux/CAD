# Publishing the frozen repository

The runtime used to prepare this repository does not expose authenticated GitHub repository-creation or `gh` CLI access. The repository is therefore delivered as a fully committed Git repository rather than a falsely reported remote repository.

## Publish

1. Create an empty GitHub repository named `aec-cad-os` under the target owner (do not initialize it with README/license/gitignore).
2. From this directory:

```bash
git remote add origin https://github.com/<OWNER>/aec-cad-os.git
git push -u origin main
```

3. Enable branch protection so the frozen-spec and quality workflows are required checks before merge.

## First implementation cycle

Give GLM 5.3 the contents of `docs/ARCHITECT-MASTER-PROMPT.md` plus `docs/work-orders/WORK-001.md`.

The implementation agent must work on a dedicated branch, return concrete evidence, and open a PR. The architect reviews that PR against `docs/reviews/ARCHITECT-REVIEW-PROTOCOL.md` and the applicable checkpoint in `docs/reviews/CHECKPOINT-PROTOCOL.md`.

Never send multiple unapproved Work Orders to the agent at once.
