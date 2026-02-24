test.host.o: test.c
	@echo "Using HOST rule"

%.o: %.c
	@echo "Using regular rule"
