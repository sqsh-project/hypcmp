import sys
import subprocess as sp
import json
from dataclasses import dataclass, field

""" Piped commands not working currently
"""

@dataclass
class Annotation:
  json: str
  data: dict = field(init=False)

  def __post_init__(self):
      with open(self.json, 'r') as f:
        self.data = json.load(f)

  def execute_cmd(self, key, cmd):
    try:
      result = sp.check_output(cmd, shell=True).decode('ascii')
    except sp.CalledProcessError as err:
      print(f"Failed w/ {err}")
      raise
    else:
      tmp = self.data["results"][0]
      if hasattr(tmp,"annotations"):
        tmp["annotations"][key] = result
      else:
        tmp["annotations"] = {key:result}
      self.data["results"][0] = tmp

  def export(self):
    with open('new_' + self.json, 'w') as f:
      json.dump(self.data, f, indent=2)


def main(keyword, hyperfine_json, cmd):
  annon = Annotation(hyperfine_json)
  annon.execute_cmd(keyword, cmd)
  annon.export()

if __name__ == '__main__':
  keyword = sys.argv[1]
  js = sys.argv[2]
  command = sys.argv[3:]
  main(keyword = keyword, hyperfine_json = js, cmd = command)
