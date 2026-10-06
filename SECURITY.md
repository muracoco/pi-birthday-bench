# Security and benchmark privacy

Documentation, GUI defaults and tests use synthetic dates. `20000101` is an
example value and is not intended to identify any person. Do not commit real
birth dates, personal benchmark results, private keys or credentials.
Benchmark output contains the requested target and may contain hardware details;
review and anonymize it before sharing. Keep local output in `benchmark-results/`.

For the maintainer's public commits, configure:

```sh
git config user.email "207765797+muracoco@users.noreply.github.com"
git var GIT_AUTHOR_IDENT
git var GIT_COMMITTER_IDENT
```

Keep GitHub email privacy and push protection enabled. Inspect staged files and
commit metadata before pushing. After a privacy history rewrite, use a fresh
clone instead of merging or force-pushing an older clone. Keep recovery backups
private. Settings and ignore rules do not remove existing published history.

Report suspected exposures privately to the maintainer. Revoke or rotate an
exposed credential before removing it from files and history.
