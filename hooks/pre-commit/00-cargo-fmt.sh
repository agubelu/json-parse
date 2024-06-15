for rustfile in $(git diff --cached --diff-filter=ACMR --name-only | grep -P "\.rs$")
do
    cargo fmt "$rustfile"
    git add "$rustfile"
done
