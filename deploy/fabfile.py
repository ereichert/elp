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
    # TODO Split the next version code into a separate method
    release(release_context)
    package()
    print("Publishing {} to crates.io.", package_path)
    publish()
    # TODO Call the next version method here passing it the release context.


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
