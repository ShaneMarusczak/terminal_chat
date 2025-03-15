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

git status
echo

git --no-pager diff
echo

echo "Continue? y/n"
read -r confirm

if [ "$confirm" != "y" ]; then
  echo "Exiting"
  exit 0
fi

git add .

# Prompt for commit message to ensure a meaningful commit
echo "Enter commit message: "
read -r commit_message
git commit -m "$commit_message"

git push
