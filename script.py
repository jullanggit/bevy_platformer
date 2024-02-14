for zahl in (100, 1000000):
    d1, d2, d3 = str(zahl)[0], str(zahl)[1], str(zahl)[2]
    if len(str(zahl)) == 3:
        if zahl % 37 == 0:
            if zahl % 11 == 0:
                if (int(d1) + int(d2) + int(d3)) > 10:
                    print(zahl)
                if sorted(str(zahl))[0] != d1 and sorted(str(zahl))[2] != d1:
                    print(zahl)
            if d3 == "0":
                if (int(d1) + int(d2) + int(d3)) > 10:
                    print(zahl)
                if sorted(str(zahl))[0] != d1 and sorted(str(zahl))[2] != d1:
                    print(zahl)
