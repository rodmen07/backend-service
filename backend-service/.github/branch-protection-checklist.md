# Branch Protection Checklist

Use this checklist when configuring protection for the `main` branch.

- [ ] Require a pull request before merging
- [ ] Require at least 1 approving review
- [ ] Dismiss stale approvals when new commits are pushed
- [ ] Require status checks to pass before merging
- [ ] Mark `CI / rust` as a required check
- [ ] Require branches to be up to date before merging
- [ ] Block force pushes
- [ ] Block branch deletion
- [ ] Restrict who can push directly to the default branch (optional, recommended)
