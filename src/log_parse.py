import itertools

for b_line, g_line in itertools.zip_longest(open('out'), open('../tests/nestest.log')):
    b_line = b_line.split()
    g_line = g_line.split()

    inst_bad = b_line[0].partition('(')[2].partition(')')[0]
    pc_bad = b_line[1].split(':')[1]
    acc_bad = b_line[2].split(':')[1]
    x_bad = b_line[3].split(':')[1]
    y_bad = b_line[4].split(':')[1]
    flag_bad = b_line[5].split(':')[1]
    sp_bad = b_line[6].split(':')[1]
    cyc_bad = b_line[7].split(':')[1]

    pc_good = g_line[0]
    acc_good = g_line[-8].split(':')[1]
    x_good = g_line[-7].split(':')[1]
    y_good = g_line[-6].split(':')[1]
    flag_good = g_line[-5].split(':')[1]
    sp_good = g_line[-4].split(':')[1]
    cyc_good = g_line[-1].split(':')[1]

    if pc_good != pc_bad:
        print("Error at " + pc_good)
        break
    elif acc_good != acc_bad:
        print("acc_good " + acc_good )
        print("acc_bad " + acc_bad )
        print("Error at " + pc_good)
        break
    elif x_good != x_bad:
        print("x_good " + x_good )
        print("x_bad " + x_bad )
        print("Error at " + pc_good)
        break
    elif y_good != y_bad:
        print("y_bad " + y_bad )
        print("Error at " + pc_good)
        break
    elif flag_good != flag_bad:
        print("flag_bad " + flag_bad )
        print("Error at " + pc_good)
        break
    elif sp_good != sp_bad:
        print("sp_bad " + sp_bad )
        print("Error at " + pc_good)
        break
    elif cyc_good != cyc_bad:
        print("cyc_bad " + cyc_bad )
        print("Error at " + pc_good)
        break
