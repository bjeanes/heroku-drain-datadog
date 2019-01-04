workflow "Run Tests" {
  on = "push"
  resolves = ["Tests"]
}

action "Build" {
  uses = "actions/docker/cli@master"
  args = "build -t drain ."
}

action "Tests" {
  uses = "actions/docker/cli@master"
  needs = ["Build"]
  args = "run drain cargo test"
}
