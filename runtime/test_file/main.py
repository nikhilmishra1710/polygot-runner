with open("large.bin", "wb") as f:
    while True:
        f.write(b"x" * 8192)