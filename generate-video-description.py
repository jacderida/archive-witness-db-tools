#!/usr/bin/env python3

import pandas as pd
import questionary
import sys

# The missing 15 is intentional. This release is not used.
NIST_RELEASE_DATES = {
    1: "2011-01-17",
    2: "2011-01-19",
    3: "2011-01-17",
    4: "2011-01-17",
    5: "2011-01-20",
    6: "2011-01-17",
    7: "2011-01-19",
    8: "2011-01-17",
    9: "2011-01-18",
    10: "2011-04-17",
    11: "2011-01-24",
    12: "2011-01-17",
    13: "2011-01-19",
    14: "2011-01-19",
    16: "2011-01-19",
    17: "2011-01-17",
    18: "2011-01-17",
    19: "2011-01-19",
    20: "2011-01-25",
    21: "2011-01-25",
    22: "2011-01-17",
    23: "2011-01-17",
    24: "2011-01-24",
    25: "2011-01-17",
    26: "2011-01-27",
    27: "2011-01-25",
    28: "2011-01-17",
    29: "2011-01-17",
    30: "2011-01-17",
    31: "2011-01-17",
    32: "2011-01-20",
    33: "2011-01-24",
    34: "2011-01-25",
    35: "2011-01-19",
    36: "2011-04-10",
    37: "2013-03-09",
    38: "2013-03-10",
    39: "2013-03-10",
    40: "2013-03-10",
    41: "2013-03-10",
    42: "2013-03-10",
}

def generate_description(spreadsheet_path):
    video_data = pd.read_excel(spreadsheet_path, engine="openpyxl", sheet_name="Videos")
    video_list = sorted(video_data["title"].tolist())
    selected_video = questionary.select("Select a video:", choices=video_list).ask()
    notes = questionary.text('Any additional notes to include? (e.g., "The audio was increased")').ask()

    selected_row = video_data[video_data["title"] == selected_video]
    master_title = selected_row.loc[selected_row.index[0], "master title"]
    video_description = selected_row.loc[selected_row.index[0], "description"]
    nist_notes = selected_row.loc[selected_row.index[0], "nist notes"]

    # For some reason, empty cells return a float NaN value.
    videographers = selected_row.loc[selected_row.index[0], "videographers"]
    if pd.isna(videographers):
        videographers = []
    else:
        videographers = videographers.split(';')
    reporters = selected_row.loc[selected_row.index[0], "reporters"]
    if pd.isna(reporters):
        reporters = []
    else:
        reporters = reporters.split(';')
    people = selected_row.loc[selected_row.index[0], "people"]
    if pd.isna(people):
        people = []
    else:
        people = people.split(';')
    jumpers = str(selected_row.loc[selected_row.index[0], "jumpers"])
    if pd.isna(jumpers):
        jumpers = []
    else:
        jumpers = jumpers.split(';')

    nist_release = 0
    nist_release_designations = []
    nist_release_refs = selected_row.loc[selected_row.index[0], "nist release refs"]
    try:
        nist_release = int(nist_release_refs)
    except ValueError:
        nist_release = int([x.split(':')[0].strip() for x in nist_release_refs.split('\n')][0])
        nist_release_designations = [x.split(':')[1].strip() for x in nist_release_refs.split('\n')]

    master_data = pd.read_excel(spreadsheet_path, engine="openpyxl", sheet_name="Master")
    selected_row = master_data[master_data["title"] == master_title]
    master_description = selected_row.loc[selected_row.index[0], "description"]

    print(master_description)
    print("---")
    print(video_description)
    if notes:
        print("---")
        print(notes)
    if len(videographers) > 0 or len(reporters) > 0 or len(people) > 0 or len(jumpers) > 0:
        print("---")
        if len(videographers) > 0:
            print(f"Videographers: {';'.join(videographers)}")
        if len(reporters) > 0:
            print(f"Reporters: {';'.join(reporters)}")
        if len(people) > 0:
            print(f"People: {';'.join(people)}")
        if len(jumpers) > 0:
            print(f"Jumper timestamps: {';'.join(jumpers)}")
        print("---")
    if len(nist_release_designations) == 0:
        print(
            "This material is sourced from NIST, who released it on "
            f"{NIST_RELEASE_DATES[nist_release]}, following an FOIA request for all photos and "
            "videos used as part of their investigation of the World Trade Center disaster. There "
            f"were 42 releases pertaining to this request. "
            f"This footage was part of release {nist_release}."
         )
    elif len(nist_release_designations) > 1:
        print(
            "This material is sourced from NIST, who released it on "
            f"{NIST_RELEASE_DATES[nist_release]}, following an FOIA request for all photos and videos "
            "used as part of their investigation of the World Trade Center disaster. There were 42 "
            f"releases pertaining to this request. This footage was part of release {nist_release}. "
            "It spanned several DVDs, with the following designations:"
         )
        for designation in nist_release_designations:
            print(f"{designation}")
    else:
        print(
            "This material is sourced from NIST, who released it on "
            f"{NIST_RELEASE_DATES[nist_release]}, following an FOIA request for all photos and videos "
            "used as part of their investigation of the World Trade Center disaster. There were 42 "
            f"releases pertaining to this request. This footage was part of release {nist_release}, "
            f"designated {nist_release_designations[0]}."
         )
    if not pd.isna(nist_notes) and nist_notes:
        print("---")
        print("Notes from NIST's database (any poor style, grammar or spelling is from them):")
        print(nist_notes)

if __name__ == "__main__":
    spreadsheet_path = sys.argv[1]
    generate_description(spreadsheet_path)
