#!/usr/bin/env python3

import os
import sys, getopt
import json

from subprocess import Popen, PIPE

def write_chao_and_chao_gran_json_file(mnemonic_word, babe_account_id, grandpa_account_id, i):
    babe_json = "chao"+str(i)+".json"
    gran_json = "chao_gran"+str(i)+".json"

    babe_dict = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "author_insertKey",
    "params": ["babe", mnemonic_word, babe_account_id]
    }
    with open(babe_json,"w") as f:
        json.dump(babe_dict, f)

    gran_dict = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "author_insertKey",
    "params": ["gran", mnemonic_word, grandpa_account_id]
    }
    with open(gran_json, "w") as f:
        json.dump(gran_dict, f)

def gen_keys(key_file: str, i) -> (str, str, str):
    sr_key1 = ''
    sr_key2 = ''
    ed_key = ''
    mnemonic_chaos = [
        "wet wait more hammer glass drastic reform detect corn resource lake bomb",
        "license trigger sight gallery trophy before rough village clean become blur blast",
        "discover despair state general virtual method ten someone rookie learn damage artefact"
    ]
    secret = ''
    mnemonic = ''
    bebe_account_id = ''
   
    if i < 3:
        secret = mnemonic_chaos[i]
        mnemonic = secret
        print(secret)
        
        output = Popen(["subkey", "inspect", secret], stdout=PIPE).communicate()[0]
        output = str(output, 'utf-8')
        with open(key_file+str(i), 'w') as fp:
            print(output, file=fp)
        output_lines = output.splitlines()
        
        sr_key1 = output_lines[4].split()[2]
        print(sr_key1)
        bebe_account_id = output_lines[3].split()[2]
    else:
        output = Popen("subkey generate".split(), stdout=PIPE).communicate()[0]
        output = str(output, 'utf-8')
        with open(key_file+str(i), 'w') as fp:
            print(output, file=fp)

        output_lines = output.splitlines()
        secret = output_lines[0].split('`')[1]
        print(secret)
        mnemonic = secret

        sr_key1 = output_lines[4].split()[2]
        print(sr_key1)

        bebe_account_id = output_lines[3].split()[2]


    ed_key_args = ["subkey", "inspect"]
    ed_key_args.append(secret + "//stash")
    output = Popen(ed_key_args, stdout=PIPE).communicate()[0]
    output = str(output, 'utf-8')
    output_lines = output.splitlines()
    sr_key2 = output_lines[4].split()[2]
    print(sr_key2)

    ed_key_args = ["subkey", "inspect", "--scheme", "ed25519"]
    ed_key_args.append(secret)
    output = Popen(ed_key_args, stdout=PIPE).communicate()[0]
    output = str(output, 'utf-8')
    output_lines = output.splitlines()
    ed_key = output_lines[4].split()[2]
    print(ed_key)

    grandpa_account_id = output_lines[3].split()[2]
    write_chao_and_chao_gran_json_file(mnemonic, bebe_account_id, grandpa_account_id, i)

    return (sr_key1, sr_key2, ed_key)

# sr_key2 is the special sr25119 key
def chain_spec_gen(input_file: str, output_file: str, new_validators,  is_sudo: bool) -> None:
    json_object = None
    key_file = "key_file"

    with open(input_file) as fp:
        json_object = json.load(fp)

    for i in range(new_validators):
        sr_key1, sr_key2, ed_key = gen_keys(key_file, i)

        # palletBalances
        json_object["genesis"]["runtime"]["palletBalances"]["balances"].append([sr_key1, 9000000000000000099999])
        json_object["genesis"]["runtime"]["palletBalances"]["balances"].append([sr_key2, 9000000000000000099999])

        # palletStaking
        if "palletStakingWithCredit" in json_object["genesis"]["runtime"]:
            # json_object["genesis"]["runtime"]["palletStakingWithCredit"]["invulnerables"].append(sr_key2)
            json_object["genesis"]["runtime"]["palletStakingWithCredit"]["stakers"].append([sr_key2, sr_key1, 10000000000099999, "Validator"])

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
    key_file = 'key_file'
    validaors_numbers = 3

    try:
        opts, args = getopt.getopt(argv,"hsi:o:k:n:",["ifile=","ofile=", "kfile=","nvalidators"])
    except getopt.GetoptError:
        print('chain_spec_gen.py -i <inputfile> -o <outputfile> -n <validatorNumbers>')
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
        elif opt in ("-n", "--nvalidators"):
            validaors_numbers = int(arg)

    print("Input file is ", input_file)
    print("Output file is ", output_file)
    print("Key file is ", key_file)

    
    chain_spec_gen(input_file, output_file, validaors_numbers-2, is_sudo)

if __name__ == "__main__":
    main(sys.argv[1:])
