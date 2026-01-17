.PHONY: help
.DEFAULT_GOAL := help

help:
	@$(MAKE) -C solana-xmr-swap help

%:
	@$(MAKE) -C solana-xmr-swap $@
