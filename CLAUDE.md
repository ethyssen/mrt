# mr-t

mr-t is a cli tool for simplifying my common workflows.

Most of the time there are a small set of decisions to be made, surrounded by many steps.
I'd like to start automating these steps so that my time and attention is freed up.

Commands:

- temp-strat: set up a temporary strategy for ticket repro and fix testing

---

The capabilities i'd like to start building now are:

Lets try to make this tickview release faster (in human time not machine time):

Tickview had a bug where it was crashing on a particular date while processing imbalances.
The source parquet was empty and replay's read_all function was panicking.
So actually, tickview doesn't even need a release.

Replay needs a release though and its a processTM:

- switch to main/master
- pull
- checkout a new branch (pick a name)
- make the edit
- choose the new version number
- go find-replace edit the version numbers
- git add .
- git commit -m "choose a msg"
- (pre-commit checks run)
- git push
- (pre-push checks run)
- click on link to create pull request
- review the code one last time (this is always a good idea!!)
- click auto-merge
- wait some amount of time
- once its in, click "New Release"
- make a new tag
- generate and edit release notes
- publish release
- wait some amount of time for it to finish

The actual human inputs/decisions and their importance:

- new branch name
- make the code change
- choose the new version number based on previously published version and breakingness of the change.
- choose commit msg so people can see and find the fix in the commit history
- second review of the code within the PR
- decide whether to release now

---

Ok how can we streamline this

branch name: this needs to not conflict with existing user branches or existing github.com branches
AND it needs to be logical for the thing we're changing.

make the code change

choose new version number:
I think all we need is to decide whether this change itself is major, minor, or patch.
Then given that info, we derive the minimum acceptable version number based on the latest release.
Then the version number we actually use here is the max(HEAD version, calculated minimum version)

commit msg can be AI generated with a human confirmation/overwrite option.
second review of the code should also be done by human
decide whether to release now is just a bool.

---

The sequence could look like this:

human.0: start a new workflow with some command
auto.0: init a working branch off of HEAD
human.1: make the code change (human or AI)
human.2: some command to "check and move to review" (code checks run here)
auto.1: run checks. If they fail, go back to human.1
auto.2: open full-screen `git status` and `git diff` for second code review.
human.3: review the code change in the presented view ^^
human.4: some command to "approve" or go back to step 1
human.5: decide: is change semver major, minor, patch?
auto.3: generate a candidate commit msg
human.6: decide: accept/overwrite generated commit msg
human.7: decude: do we want to release now?
auto.4: everything else...
auto.z: confirm release succeeded.

---
