OBJDUMP ?= objdump
PYTHON ?= python3

.PRECIOUS: %.o

all: 00-add-r.hex 01-add-i.hex 02-sll-r.hex 03-sll-i.hex \
     04-srl-r.hex 05-srl-i.hex 06-sd-ld.hex 07-sw-lw.hex \
     08-jal.hex 09-j.hex 10-beq.hex 11-blt.hex 12-bge.hex \
     13-mul.hex 14-lui.hex 15-sb-lb.hex 16-quadratic.hex \
     17-get-bitseq.hex 18-max3.hex 19-fib-rec.hex 20-is-pal.hex \
     21-to-upper.hex 22-sort.hex 23-str-to-int.hex

%.o: %.s
	$(AS) -o $@ $<

%.hex: %.o
	$(OBJDUMP) -d $< | $(PYTHON) makerom3.py > $@

clean:
	rm -rf *.o *.hex
