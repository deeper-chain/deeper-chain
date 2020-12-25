#!/usr/bin/env python3

import os
import sys, getopt
import json

from subprocess import Popen, PIPE


def derive_addrs_from_secret(secret: str) -> (str, str, str):
    key_args = ["subkey", "inspect"]
    key_args.append(secret)
    output = Popen(key_args, stdout=PIPE).communicate()[0]
    output = str(output, 'utf-8')
    output_lines = output.splitlines()
    sr_key1 = output_lines[4].split()[2]
    print(sr_key1)

    key_args = ["subkey", "inspect"]
    key_args.append(secret + "//stash")
    output = Popen(key_args, stdout=PIPE).communicate()[0]
    output = str(output, 'utf-8')
    output_lines = output.splitlines()
    sr_key2 = output_lines[4].split()[2]
    print(sr_key2)

    key_args = ["subkey", "inspect", "--scheme", "ed25519"]
    key_args.append(secret)
    output = Popen(key_args, stdout=PIPE).communicate()[0]
    output = str(output, 'utf-8')
    output_lines = output.splitlines()
    ed_key = output_lines[4].split()[2]
    print(ed_key)

    return (sr_key1, sr_key2, ed_key)


def import_addrs(key_file: str) -> (str, str, str):
    with open(key_file, 'r') as fp:
        inputs = fp.read()
        input_lines = inputs.splitlines()
        secret = input_lines[1].split()[2]

    return derive_addrs_from_secret(secret)



def gen_keys(key_file: str) -> (str, str, str):
    sr_key1 = ''
    sr_key2 = ''
    ed_key = ''

    output = Popen("subkey generate".split(), stdout=PIPE).communicate()[0]
    output = str(output, 'utf-8')

    with open(key_file, 'w') as fp:
        print(output, file=fp)

    output_lines = output.splitlines()
    secret = output_lines[0].split('`')[1]

    return derive_addrs_from_secret(secret)

# sr_key2 is the special sr25119 key
def chain_spec_gen(input_file: str, output_file: str, sr_key1: str, sr_key2: str, ed_key: str, is_sudo: bool) -> None:
    json_object = None

    with open(input_file) as fp:
        json_object = json.load(fp)

    # palletBalances
    json_object["genesis"]["runtime"]["palletBalances"]["balances"].append([sr_key1, 9000000000000000099999])
    json_object["genesis"]["runtime"]["palletBalances"]["balances"].append([sr_key2, 9000000000000000099999])

    # palletStaking
    if "palletStaking" in json_object["genesis"]["runtime"]:
       # json_object["genesis"]["runtime"]["palletStaking"]["invulnerables"].append(sr_key2)
        json_object["genesis"]["runtime"]["palletStaking"]["stakers"].append([sr_key2, sr_key1, 10000000000099999, "Validator"])


    # palletSession
    if "palletSession" in json_object["genesis"]["runtime"]:
        json_object["genesis"]["runtime"]["palletSession"]["keys"].append([sr_key2, sr_key2, {"grandpa": ed_key, "babe": sr_key1, "im_online": sr_key1, "authority_discovery": sr_key1}])

    # palletCollectiveInstance2
    if "palletCollectiveInstance2" in json_object["genesis"]["runtime"]:
        json_object["genesis"]["runtime"]["palletCollectiveInstance2"]["members"].append(sr_key1)

    # palletElectionsPhragmen
    if "palletElectionsPhragmen" in json_object["genesis"]["runtime"]:
        json_object["genesis"]["runtime"]["palletElectionsPhragmen"]["members"].append([sr_key1, 10000000000099999])

    # palletSudo
    if is_sudo == True:
        json_object["genesis"]["runtime"]["palletSudo"]["key"] = sr_key1

    # palletSociety
    if "palletSociety" in json_object["genesis"]["runtime"]:
        json_object["genesis"]["runtime"]["palletSociety"]["members"].append(sr_key1)

    with open(output_file, "w") as fp:
        json.dump(json_object, fp, indent=4)


def main(argv):
    input_file = ''
    output_file = ''
    is_sudo = False
    key_file = ''

    try:
        opts, args = getopt.getopt(argv,"hsi:o:k:",["ifile=","ofile=", "kfile="])
    except getopt.GetoptError:
        print('chain_spec_gen.py -i <inputfile> -o <outputfile>')
        sys.exit(2)
    for opt, arg in opts:
        if opt == '-h':
            print('chain_spec_gen.py -i <inputfile> -o <outputfile>')
            sys.exit()
        elif opt in ("-i", "--ifile"):
            input_file = arg
        elif opt in ("-o", "--ofile"):
            output_file = arg
        elif opt == '-s':
            is_sudo = True
        elif opt in ("-k", "--kfile"):
            key_file = arg

    print("Input file is ", input_file)
    print("Output file is ", output_file)

    if key_file == '':
        print("Key file is ", key_file)
        sr_key1, sr_key2, ed_key = gen_keys('key_file')
    else:
        sr_key1, sr_key2, ed_key = import_addrs(key_file)
    chain_spec_gen(input_file, output_file, sr_key1, sr_key2, ed_key, is_sudo)

if __name__ == "__main__":
    main(sys.argv[1:])
