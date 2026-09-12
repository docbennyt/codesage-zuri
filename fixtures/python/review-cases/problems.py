import subprocess

def mutable(items=[]):
    return items

def risky(code):
    return eval(code)

def too_broad():
    try:
        return 1 / 0
    except:
        return None

def shell(command):
    return subprocess.run(command, shell=True)
