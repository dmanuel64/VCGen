import sys
import pandas as pd

if len(sys.argv) != 2:
    print('Usage: inspect.py <DATASET_JSONL>', file=sys.stderr)
    exit(1)


def inspect_code(code: str, ff_report: str):
    ff_report = ff_report.split('ANALYSIS SUMMARY')[0]
    lines = code.splitlines()
    for vulnerability in ff_report.split('/tmp/')[1:]:
        vulnerability_idx = int(ff_report.split(':')[2]) - 1
        print('\n' + vulnerability + '\n')
        if vulnerability_idx > 1:
            print(lines[vulnerability_idx - 1])
        print(lines[vulnerability_idx])
        if vulnerability_idx < len(lines) - 1:
            print(lines[vulnerability_idx + 1])
        input('\nPress anything to continue to the next vulnerability')


df = pd.read_json(sys.argv[1], lines=True)
df.apply(lambda row:
         inspect_code(row['Code'], row['Flawfinder Vulnerabilities']), axis=1)
