from pathlib import Path
import pandas as pd
import subprocess
import sys


VCGEN_PATH = Path.cwd().joinpath('target/debug/vcgen')

if len(sys.argv) < 3:
    subprocess.run([VCGEN_PATH, '-h'])
else:
    jsonl_file_path = \
        Path(list(filter(lambda arg: arg.endswith('jsonl'), sys.argv))[0])

    if subprocess.run([VCGEN_PATH, *sys.argv[1:]]).returncode == 0 and jsonl_file_path.exists():
        df = pd.read_json(jsonl_file_path, lines=True)
        df.to_excel(jsonl_file_path.parent.absolute().joinpath(jsonl_file_path.stem + '.xlsx'),
                    engine='xlsxwriter')
