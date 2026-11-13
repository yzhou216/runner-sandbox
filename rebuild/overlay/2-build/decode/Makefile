PYTHON ?= python3

all: decode.hex

%.hex: %.bin
	cat decode.bin | $(PYTHON) mkdecoderom.py > decode.hex

clean:
	rm -rf *.hex
