from setuptools import setup, find_namespace_packages
from setuptools_rust import Binding, RustExtension


setup(
    name="ephemeris",
    version="0.1.0",
    packages=["ephemeris"],
    zip_safe=False,
    rust_extensions=[
        RustExtension(
            "ephemeris.ephemeris",
            path="Cargo.toml",
            debug=False,
        )
    ],
)