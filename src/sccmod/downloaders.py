class GitClone:
    def __init__(self, url, branch=None, commit=None, tag=None, submodules=True, shallow=False):
        self.url = url
        self.branch = branch
        self.commit = commit
        self.submodules = submodules
        self.shallow = shallow

class Curl:
    def __init__(self, url, archive=None, sha256=None):
        self.url = url
        self.archive = archive
        self.sha256 = sha256
