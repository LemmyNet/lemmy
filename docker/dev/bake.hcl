variable "V" {
    default = "latest"
}

group "default" {
    targets = ["lemmy-x64", "lemmy-arm"]
}

target "lemmy-x64" {
    dockerfile = "docker/dev/Dockerfile"
    context = "../.."
    platforms = ["linux/amd64"]
    tags = ["docker.io/shtripok/lemmy:${V}"]
}

target "lemmy-arm" {
    inherits = ["lemmy-x64"]
    dockerfile = "docker/dev/Dockerfile.libc"
    platforms = [ "linux/arm64"]
}

