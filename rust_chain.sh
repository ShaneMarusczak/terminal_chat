#!/bin/bash

set -e

echo "fmt"
cargo fmt
echo

echo "test"
cargo test
echo

echo "check"
cargo check
echo

echo "clippy"
cargo clippy
echo

echo "machete"
cargo machete
echo

git_status=$(git status --porcelain)
if [[ -n "$git_status" ]]; then
  echo "Changes detected in git status."
else
  echo "No changes to commit."
  exit 0
fi

echo
git --no-pager diff
echo

echo "Would you like to add changes? y/n"
read -r add_confirm

if [ "$add_confirm" != "y" ]; then
  echo "Exiting"
  exit 0
fi

git add .

echo "Continue with commit? y/n"
read -r continue_confirm

if [ "$continue_confirm" != "y" ]; then
  echo "Exiting"
  exit 0
fi

echo "Enter commit message: "
read -r commit_message

if [[ -z "$commit_message" ]]; then
  echo "Commit message cannot be empty."
  exit 1
fi

git commit -m "$commit_message"
git push
