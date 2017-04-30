#!/usr/bin/env python

from distutils.util import strtobool

from git import Repo
import contoml
import semantic_version
import shutil

import fabric
from fabric.api import local, task
from fabric.decorators import runs_once

from deploy_common import *
from build import *

RELEASE_TYPE_SNAPSHOT = "snapshot"
RELEASE_TYPE_FINAL = "final"
RELEASE_TYPE_TEST_FINAL = "testfinal"  # used to test final releases without committing to master.
RELEASE_TYPES = [RELEASE_TYPE_SNAPSHOT, RELEASE_TYPE_FINAL, RELEASE_TYPE_TEST_FINAL]
SNAPSHOT = "SNAPSHOT"
BRANCH_DEVELOP = "develop"
BRANCH_MASTER = "master"
BRANCH_TEST_MASTER = "testmaster"
BRANCH_TEST_DEVELOP = "testdevelop"


@runs_once
@task
def release(release_context):
    if not release_context.dry_run:
        print("*** You are about to do a release. This is not a dry run. ***")
        resp = _prep_bool_arg(raw_input('Confirm [Y/N]: '))
        if not resp:
            fabric.utils.abort("Release aborted.")

    if release_context.release_type not in RELEASE_TYPES:
        fabric.utils.abort("You must specify the release type: [snapshot xor final xor testfinal]")

    if not release_context.disable_checks and release_context.repo_active_branch().lower() != BRANCH_DEVELOP:
        fabric.utils.abort("You must be on the develop branch in order to do a release. You are on branch {}".format(
            release_context.repo_active_branch()))

    if not release_context.disable_checks and release_context.repo_is_dirty():
        fabric.utils.abort("There are uncommitted changes on the develop branch.")

    if release_context.is_test_final_release():
        release_context.checkout_test_develop()

    starting_version, package_name = read_cargo_file(release_context)
    print("starting version = {sv}, package name = {pn}".format(sv=starting_version, pn=package_name))

    release_version = confirm_version(release_context, semantic_version.Version(starting_version))
    print "Releasing {} v{}".format(package_name, str(release_version))

    update_version_in_files(release_context, release_version)

    if release_context.run_build:
        build_and_test(PROJECT_ROOT)

    print "Build and tests completed successfully"
    if not release_context.dry_run:
        release_context.commit_release("Release commit for {}.".format(str(release_version)))

    print "Committed release v{} to {}.".format(str(release_version), release_context.repo_active_branch())

    tag = "{}-{}".format(package_name, str(release_version))
    if not release_context.dry_run:
        release_context.tag_release(tag, tag)

    print "Tagged release v{} to {}.".format(str(release_version), release_context.repo_active_branch())

    if not release_context.dry_run:
        print "Pushing release to origin."
        release_context.push_to_origin()


@runs_once
@task
def bump_version(release_context):
    starting_version, package_name = read_cargo_file(release_context)

    if release_context.is_snapshot_release():
        snapshot_version = to_snapshot_version(starting_version)
        update_version_in_files(release_context, snapshot_version)
        print("Updated files with SNAPSHOT specifier.")
        if not release_context.dry_run:
            release_context.commit_release("Rewrite version to SNAPSHOT.")

    if release_context.is_test_final_release():
        release_context.checkout_test_master()
        release_context.merge_test_develop()
        release_context.checkout_test_develop()
        next_version = to_next_patch_snapshot_version(starting_version)
        update_version_in_files(release_context, next_version)
        print("Updated files with SNAPSHOT specifier.")
        if not release_context.dry_run:
            release_context.commit_release("Bumped version to {}.".format(next_version))

    if release_context.is_final_release():
        release_context.checkout_master()
        release_context.merge_develop()
        release_context.checkout_develop()
        next_version = to_next_patch_snapshot_version(starting_version)
        update_version_in_files(release_context, next_version)
        print("Updated files with SNAPSHOT specifier.")
        if not release_context.dry_run:
            release_context.commit_release("Bumped version to {}.".format(next_version))

    if not release_context.dry_run:
        print "Pushing release to origin."
        release_context.push_to_origin()


class ReleaseContext:
    def __init__(
            self,
            repo_path,
            release_type,
            cargo_file,
            version_file,
            readme_file,
            disable_checks,
            dry_run,
            run_build
    ):
        # Either final or snapshot
        self.release_type = release_type.lower()
        # This should be the path to the Cargo.toml file.
        self.cargo_file = cargo_file
        # This should be the path to the version.txt file.
        self.version_file = version_file
        # This should be the path to the README.md file.
        self.readme_file = readme_file
        # disable_checks is useful for testing of the release script.
        # It should not be used normally.
        self.disable_checks = disable_checks
        # Do everything non destructively.  That is, the script will run with
        # output but nothing will actually be committed.
        self.dry_run = dry_run
        # The git repo.
        self._repo = Repo(repo_path)
        # Run the build and test processes?
        self.run_build = run_build

    def repo_active_branch(self):
        return self._repo.active_branch.name

    def repo_is_dirty(self):
        return self._repo.is_dirty()

    def commit_release(self, message):
        self._repo.git.add(update=True)
        self._repo.index.commit(message)

    def tag_release(self, tag, tag_message):
        self._repo.create_tag(tag, message=tag_message)

    def push_to_origin(self):
        self._repo.remotes.origin.push('refs/heads/*:refs/heads/*', tags=True)

    def is_snapshot_release(self):
        return self.release_type == RELEASE_TYPE_SNAPSHOT

    def is_final_release(self):
        return self.release_type == RELEASE_TYPE_FINAL

    def is_test_final_release(self):
        return self.release_type == RELEASE_TYPE_TEST_FINAL

    def checkout_master(self):
        self._repo.heads.master.checkout()

    def checkout_test_master(self):
        self.checkout_test_branch(BRANCH_TEST_MASTER)

    def checkout_test_develop(self):
        self.checkout_test_branch(BRANCH_TEST_DEVELOP)

    def checkout_test_branch(self, branch_name):
        if branch_name in self._repo.heads:
            self._repo.delete_head(branch_name, ["-D"])

        self._repo.create_head(branch_name)
        self._repo.heads[branch_name].checkout()

    def checkout_develop(self):
        self._repo.heads.develop.checkout()

    def merge_develop(self):
        self._repo.git.merge(BRANCH_DEVELOP)

    def merge_test_develop(self):
        self._repo.git.merge(BRANCH_TEST_DEVELOP)


def build_and_test(project_root):
    try:
        build(project_root)
        test(project_root)
    except BuildException as e:
        fabric.utils.abort("{}".format(e.value))
    except TestsFailedException as e:
        fabric.utils.abort("{}".format(e.value))


def read_cargo_file(release_context):
    with open(release_context.cargo_file) as cargo_file:
        cargo_content = contoml.loads(cargo_file.read())
        return cargo_content['package']['version'], cargo_content['package']['name']


def confirm_version(release_context, current_version):
    confirmed_version = None
    presentation_version = to_presentation_version(release_context, current_version)
    while confirmed_version is None:
        # We confirm current_version if the user does not specify a version
        # because current_version may not be valid for the type of release the
        # user specified.
        input_version = raw_input('Set version [{}]: '.format(presentation_version)) or str(presentation_version)
        confirmed_version = is_valid_proposed_version(release_context, input_version)
        if confirmed_version is None:
            print("{} does not fit the semantic versioning spec or is not valid given the specified release type of {}."
                  .format(input_version, release_context.release_type))

    if release_context.is_snapshot_release():
        return to_snapshot_release_version(confirmed_version)
    elif release_context.is_test_final_release():
        return to_test_final_release_version(confirmed_version)
    else:
        return confirmed_version


def to_presentation_version(release_context, version):
    if release_context.is_snapshot_release():
        return to_snapshot_version(version)
    else:
        return to_final_release_version(version)


def is_valid_proposed_version(release_context, proposed_version):
    valid_version = False
    sv = None
    if semantic_version.validate(proposed_version):
        sv = semantic_version.Version(proposed_version)
        if release_context.is_snapshot_release():
            valid_version = sv.prerelease and sv.prerelease[0].upper() == SNAPSHOT
        else:
            valid_version = not sv.prerelease

    if valid_version:
        return sv
    else:
        return None


def to_next_patch_snapshot_version(original_version):
    return semantic_version.Version(
        '{}.{}.{}-{}'.format(
            original_version.major,
            original_version.minor,
            original_version.patch + 1,
            SNAPSHOT
        )
    )


def to_snapshot_version(original_version):
    return semantic_version.Version(
        '{}.{}.{}-{}'.format(
            original_version.major,
            original_version.minor,
            original_version.patch,
            SNAPSHOT
        )
    )


def to_snapshot_release_version(original_version):
    return semantic_version.Version(
        '{}.{}.{}-{}'.format(
            original_version.major,
            original_version.minor,
            original_version.patch,
            local("git rev-parse --short HEAD", capture=True)
        )
    )


def to_test_final_release_version(original_version):
    return semantic_version.Version(
        '{}.{}.{}-{}'.format(
            original_version.major,
            original_version.minor,
            original_version.patch,
            'TESTFINALRELEASE'
        )
    )


def to_final_release_version(original_version):
    return semantic_version.Version(
        '{}.{}.{}'.format(
            original_version.major,
            original_version.minor,
            original_version.patch
        )
    )


def update_version_in_files(release_context, version):
    version_string = str(version)
    update_cargo_file_version(release_context, version_string)
    print 'Updated {} with the release version.'.format(release_context.cargo_file)


def update_cargo_file_version(release_context, version):
    with open(release_context.cargo_file, 'r+') as cargo_file:
        cargo_content = contoml.loads(cargo_file.read())
        cargo_content['package']['version'] = version
        cargo_content.dump(release_context.cargo_file)


def _prep_bool_arg(arg):
    return bool(strtobool(str(arg)))
