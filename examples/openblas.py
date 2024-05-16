from sccmod.downloaders import GitClone
from sccmod.builders import CMake
from sccmod import Module


class OpenBLAS:
    def __init__(self, compiler, parallel, sha=None):
        self.parallel = parallel
        self.compiler = compiler
        self.sha = sha

    def metadata(self):
        id = f"libraries/blas/OpenBLAS/{"parallel" if self.parallel else "serial"}/latest/{self.compiler}"

        return {
            "identifier": id,
            "name": "OpenBLAS",
            "description": "Optimised BLAS implementation for CPU",
            "version": "latest",
            "parallel": str(self.parallel),
            # Download and install path helpers
            "download_path": "openblas-latest",
            "build_path": "openblas-latest",
        }

    def download(self):
        return GitClone("https://github.com/OpenMathLib/OpenBLAS.git", commit=self.sha)

    def build_requirements(self):
        return [Module(self.compiler)] + ([Module("openmp")] if self.parallel else [])

    def build(self):
        return CMake(
            build_type="Debug",
            jobs=12,
            configure_flags=[
                "-DCMAKE_MT=mt",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DBUILD_WITHOUT_LAPACK=OFF",
                "-DNOFORTRAN=0",
                "-DBINARY=64",
                "-DUSE_OPENMP=" + ("ON" if self.parallel else "OFF"),
                "-DUSE_THREAD=" + ("ON" if self.parallel else "OFF"),
                "-DTARGET=AARCH64",
                "-DCMAKE_BUILD_TYPE=Release",
            ],
        )

    def __repr__(self):
        return f"OpenBLAS(compiler={self.compiler}, sha={self.sha})"


def generate():
    return [
        # OpenBLAS(Module("gcc-11"), parallel=True),
        # OpenBLAS(Module("gcc-12"), parallel=True),
        OpenBLAS("gcc-13", parallel=True),
    ]
