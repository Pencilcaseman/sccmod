class CMake:
    def __init__(
        self,
        build_type="Release",
        jobs=8,
        prefix_args=None,
        configure_flags=None,
        cmake_root=None,
    ):
        self.build_type = build_type
        self.jobs = jobs
        self.prefix_args = prefix_args or []
        self.configure_flags = configure_flags or []
        self.cmake_root = cmake_root


class Make:
    def __init__(
        self,
        configure=True,
        jobs=8,
        prefix_args=None,
        configure_flags=None,
        make_root=None,
    ):
        self.configure = configure
        self.jobs = jobs
        self.prefix_args = prefix_args or []
        self.configure_flags = configure_flags or []
        self.make_root = make_root
