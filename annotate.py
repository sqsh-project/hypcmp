import sys
import subprocess as sp


def main(keyword, hyperfine_json, cmd):
  result = execute_cmd(cmd)
  print(keyword, hyperfine_json, cmd, result)


def execute_cmd(cmd):
  try:
    result = sp.check_output(cmd, shell=True)
  except sp.CalledProcessError as err:
    print(f"Failed w/ {err}")
    raise
  else:
    return result


if __name__ == '__main__':
  keyword = sys.argv[1]
  json = sys.argv[2]
  command = sys.argv[3:]
  main(keyword = keyword, hyperfine_json = json, cmd = command)
