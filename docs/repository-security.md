---
title: Repository security
type: operations
status: current
updated: 2026-09-05
sources:
  - .github/workflows/release.yml
  - docs/release-process.md
---

# Repository security

## Protected branches

The `main` branch is protected in both `gamedacctl` and `omarchy-gamedacctl`.
The policy applies to the repository administrator and requires:

- changes to arrive through a pull request;
- the branch to be current before merging;
- the repository's required GitHub Actions check to pass;
- all review conversations to be resolved; and
- linear history.

Force-pushes and deletion of `main` are disabled. Zero approving reviews are
required because the repositories currently have one maintainer; this preserves
the pull-request and CI gate without making the maintainer unable to merge their
own work. Increase the approval count when another regular maintainer is
available.

The required controller check is **Validate and build Arch artifacts**. The
required Omarchy adapter check is **Validate Omarchy plugin**. The latter runs
the adapter's `scripts/validate`, while a local Omarchy installation adds the
upstream `omarchy plugin validate` check automatically.

## Normal change workflow

Create a topic branch, validate locally, commit, push that branch, and open a
pull request. Merge only after the required check succeeds and GitHub reports
the branch mergeable. Delete the topic branch after merging. Direct pushes to
`main` are intentionally rejected, including pushes by the owner.

Release tags are not matched by this branch-scoped policy. Follow
[the release process](release-process.md), which requires the version change to
merge through a protected pull request and the exact merged commit to pass its
main-branch dry run before an immutable annotated tag is pushed.

## Audit and recovery

The current live policy is visible in the branch settings for
[`gamedacctl`](https://github.com/AndreasDellrud/gamedacctl/settings/branches)
and
[`omarchy-gamedacctl`](https://github.com/AndreasDellrud/omarchy-gamedacctl/settings/branches),
and through GitHub's branch-protection API. Treat changes to the policy itself
as a security-sensitive operation and record the reason in Beads and this log.

If a required check is renamed or removed, update branch protection only after
the replacement check has completed successfully on the default branch. Do not
disable administrator enforcement merely to bypass a failing change; repair the
change or the check on a topic branch.
