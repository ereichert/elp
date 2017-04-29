from fabric.decorators import runs_once
from fabric.decorators import task

import subprocess


@runs_once
@task
def build(project_root):
    try:
        print "********** RUNNING THE BUILD ********"
        build_cmd = "cd {src_dir} && cargo clean && UPDATE_BUILD_INFO=1 cargo build --release".format(src_dir=project_root)
        ret_code = subprocess.call(build_cmd, shell=True)
        if ret_code != 0:
            raise BuildException("The build failed. See build output for more information.")
    except OSError as e:
        raise BuildException(e.value)


@runs_once
@task
def test(project_root):
    try:
        print "********** RUNNING TESTS ********"
        test_cmd = "cd {src_dir} && cargo test --release".format(src_dir=project_root)
        ret_code = subprocess.call(test_cmd, shell=True)
        if ret_code != 0:
            raise TestsFailedException("Tests failed. See test output for more information.")
    except OSError as e:
        raise TestsFailedException(e.value)


class BuildException(Exception):

    def __init__(self, value):
        self.value = value

    def __str__(self):
        return repr(self.value)


class TestsFailedException(Exception):

    def __init__(self, value):
        self.value = value

    def __str__(self):
        return repr(self.value)
