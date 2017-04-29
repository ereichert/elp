import os

DEPLOYMENT_WORKING_DIR = os.getcwd()
print("DEPLOYMENT_WORKING_DIR = {}".format(DEPLOYMENT_WORKING_DIR))

PROJECT_ROOT = os.path.dirname(DEPLOYMENT_WORKING_DIR)
print("PROJECT_ROOT = {}".format(PROJECT_ROOT))


def result_handler(result, message, return_code=0):
    if result.failed or result.return_code != return_code:
        print result
        raise Exception()
    else:
        print message
