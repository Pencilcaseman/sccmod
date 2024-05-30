class Class:
    def __init__(self, name):
        self.name = name

    def __str__(self):
        return self.name

    def __repr__(self):
        return self.name

class Deny:
    def __init__(self, name):
        if isinstance(name, (list, tuple)):
            self.name = ":".join(name)
        else:
            self.name = name

    def __str__(self):
        return str(self.name.split(":"))

    def __repr__(self):
        return str(self.name.split(":"))
