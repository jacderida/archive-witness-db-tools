#!/usr/bin/env python3

import sys
import datetime

def process_timestamps(file_path, start_time):
    start_time = datetime.datetime.strptime(start_time, "%H%M")
    directory, filename = file_path.rsplit('/', 1)
    output_file_path = f"{directory}/{filename.split('.')[0]}.times.txt"
    
    with open(file_path, 'r') as file, open(output_file_path, 'w') as output:
        for line in file:
            timestamp = line[:8]
            description = line[10:]
            h, m, s = map(int, timestamp.split(':'))
            delta = datetime.timedelta(hours=h, minutes=m, seconds=s)
            new_time = start_time + delta
            formatted_time = new_time.strftime("[%H%M]")
            output.write(f"{timestamp}: {description.strip()} {formatted_time}\n")

if __name__ == "__main__":
    file_path = sys.argv[1]
    start_time = sys.argv[2]
    process_timestamps(file_path, start_time)
