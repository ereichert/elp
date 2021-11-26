## deployment
Deployment is handled by the python tool http://www.fabfile.org/.

You will create a virtual environment using virtualenv http://docs.python-guide.org/en/latest/dev/virtualenvs/.

From the ELP source code directory.

```
cd deploy
virtualenv fab_files
source elp_deploy/bin/activate (you can deactivate using the deactivate command)
```

Use the requirements.txt file located in the ../deploy directory to install all needed dependencies.

```
pip install -r requirements.txt
```

Ensure git is configured with your information.

```
git config --global --get-regexp 'user.*'
```

Once your virtual environment is installed run fabric tasks as follows.

```
fab <task>:<task_params>
```

### Releases

The two main release tasks are aliases for more complicated commands that can be used for testing the scripts.
In general, you should use the release scripts to run the fab tasks that manage a release.
To run a release you must be on the develop branch and it must be clean.

```
fab release_snapshot xor fab release_final
```

However, to run a release manually, for testing script updates for example, you can use the following command format.

```
fab release:release_type=<snapshot ^ final ^ testfinal>,<disable_checks=True>,<dry_run=False>
```

Most testing should be done using testfinal.
