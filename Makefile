# tvty — everyday targets. `make help` lists them.
.PHONY: help build run check wbox-up wbox-down wbox-restart wbox-shot wbox-click wbox-key wbox-type wbox-log fake-up sim-up flood-up fake-down

help:
	@echo "make build            cargo build (debug)"
	@echo "make run              run tvty on your own desktop"
	@echo "make check            build + launch in wbox + screenshot (what an agent reruns)"
	@echo ""
	@echo "wbox (tvty in a nested compositor, offscreen by default; WBOX_VISIBLE=1 for a window):"
	@echo "make wbox-up / wbox-down / wbox-restart"
	@echo "make wbox-shot [NAME=..]  screenshot into dev/tvty-wbox/screenshots/"
	@echo "make wbox-click X=.. Y=.. click at a position (1280x800 screen)"
	@echo "make wbox-key K=ctrl+t    send a shortcut"
	@echo "make wbox-type T=hello     type text (ASCII)"
	@echo "make wbox-log             last lines of the compositor/app log"
	@echo ""
	@echo "something to attach to, no tokens (tmux sessions tvty-fake / tvty-sim):"
	@echo "make fake-up              fake-claude in tmux session tvty-fake"
	@echo "make sim-up               simai-cli replaying dev/sim/dense.txt in tvty-sim"
	@echo "make flood-up             tvty-flood: 5 s of lines at full speed on Enter"
	@echo "make fake-down            kill them all"

build:
	cargo build

run: build
	./target/debug/tvty

# ── wbox ─────────────────────────────────────────────────────────────────────
# Same instance the tvty-wbox MCP tools drive (same config, same name), so both
# can be mixed. wbox lives in its own venv, hence WBOX_PYTHON.
WBOX_PYTHON ?= $(shell head -1 "$$(command -v wbox-mcp 2>/dev/null)" 2>/dev/null | sed -n 's/^\#!//p')
WBOX_CONFIG := dev/tvty-wbox/config.yaml
WBOX_CTL    = $(WBOX_PYTHON) scripts/wbox_ctl.py
NAME ?= shot

wbox-up:
	$(WBOX_CTL) up $(WBOX_CONFIG)

wbox-down:
	$(WBOX_CTL) down $(WBOX_CONFIG)

# Never build over a running binary: stop, build, start.
wbox-restart: wbox-down build wbox-up

wbox-shot:
	$(WBOX_CTL) shot $(WBOX_CONFIG) --name $(NAME)

wbox-click:
	$(WBOX_CTL) click $(WBOX_CONFIG) $(X) $(Y)

wbox-key:
	$(WBOX_CTL) key $(WBOX_CONFIG) $(K)

wbox-type:
	$(WBOX_CTL) type $(WBOX_CONFIG) "$(T)"

wbox-log:
	@tail -n 50 dev/tvty-wbox/log/*.log 2>/dev/null || echo "no log yet"

check: wbox-restart
	@sleep 2
	$(MAKE) wbox-shot NAME=check
	$(MAKE) wbox-down

# ── fake sessions ────────────────────────────────────────────────────────────
fake-up:
	scripts/fake-loop fake

sim-up:
	scripts/fake-loop sim

flood-up:
	scripts/fake-loop flood

fake-down:
	-scripts/fake-loop stop tvty-fake
	-scripts/fake-loop stop tvty-sim
	-scripts/fake-loop stop tvty-flood
