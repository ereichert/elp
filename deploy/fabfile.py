from build import *
from release import *
from fabric.network import ssh
from fabric.api import env, put, run, sudo, task
from fabric.decorators import runs_once

ssh.util.log_to_file("paramiko.log", 10)
env.use_ssh_config = True

WORKSPACE_DIR = os.path.join(DEPLOYMENT_WORKING_DIR, "templates/workspace/")
print("WORKSPACE_DIR = {}".format(WORKSPACE_DIR))
VALID_MODES = ["full", "dryrun"]


@runs_once
@task
def release_final():
    run_release(RELEASE_TYPE_FINAL)


@runs_once
@task
def release_snapshot():
    run_release(RELEASE_TYPE_SNAPSHOT)

@runs_once
@task
def release_test_final():
    release_context = ReleaseContext(
        PROJECT_ROOT,
        RELEASE_TYPE_TEST_FINAL,
        "{}/Cargo.toml".format(PROJECT_ROOT),
        "{}/src/version.txt".format(PROJECT_ROOT),
        "{}/CHANGELOG.md".format(PROJECT_ROOT),
        True,
        True,
        True
    )
    release(release_context)
    bump_version(release_context)


def run_release(release_type):
    release_context = ReleaseContext(
        PROJECT_ROOT,
        release_type,
        "{}/Cargo.toml".format(PROJECT_ROOT),
        "{}/src/version.txt".format(PROJECT_ROOT),
        "{}/CHANGELOG.md".format(PROJECT_ROOT),
        False,
        False,
        True
    )
    release(release_context)
    package()
    print("Publishing to crates.io.")
    publish()
    bump_version(release_context)


@runs_once
@task
def package():
    print "********** Packaging for crates.io. ********"
    package_cmd = 'cargo package'
    ret_code = subprocess.call(package_cmd, shell=True)
    if ret_code != 0:
        fabric.utils.abort("Packaging for crates.io failed with return code {}".format(ret_code))


@runs_once
@task
def publish():
    print "********** Publishing to crates.io. ********"
    publish_cmd = 'cargo publish'
    ret_code = subprocess.call(publish_cmd, shell=True)
    if ret_code != 0:
        fabric.utils.abort("Publishing to crates.io failed with return code {}".format(ret_code))
