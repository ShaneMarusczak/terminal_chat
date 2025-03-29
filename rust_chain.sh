#!/bin/bash

set -e

cargo fmt
echo
cargo check
echo
cargo test
echo
cargo clippy
echo
cargo machete
echo

git status
echo

echo "Continue? y/n -- will add . and commit"
read -r confirm

if [ "$confirm" != "y" ]; then
  echo "Exiting"
  exit 0
fi

git add .

git --no-pager diff
echo

echo "Enter commit message: "
read -r commit_message
git commit -m "$commit_message"

git push
