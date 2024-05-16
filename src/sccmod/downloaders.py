class GitClone:
    def __init__(self, url, branch=None, commit=None, tag=None, submodules=False):
        self.url = url
        self.branch = branch
        self.commit = commit
        self.submodules = submodules


class Curl:
    def __init__(self, url, sha256=None):
        self.url = url
        self.sha256 = sha256
