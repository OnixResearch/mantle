from setuptools import Extension, setup

setup(ext_modules=[Extension("mantle_native_answer", ["answer.c"])])
