# tvty — everyday targets. `make help` lists them.
.PHONY: emoji-font install-desktop uninstall-desktop help build run check public-check wbox-up wbox-down wbox-restart wbox-shot wbox-click wbox-key wbox-type wbox-scroll wbox-log fake-up sim-up flood-up fake-down aiball-up aiball-down

help:
	@echo "make build            cargo build (debug)"
	@echo "make run              run tvty on your own desktop"
	@echo "make check            build + launch in wbox + screenshot (what an agent reruns)"
	@echo "make public-check     nothing private in the tracked files (paths, ticket ids)"
	@echo ""
	@echo "wbox (tvty in a nested compositor, offscreen by default; WBOX_VISIBLE=1 for a window):"
	@echo "make wbox-up / wbox-down / wbox-restart"
	@echo "make wbox-shot [NAME=..]  screenshot into dev/tvty-wbox/screenshots/"
	@echo "make wbox-click X=.. Y=.. click at a position (1280x800 screen)"
	@echo "make wbox-key K=ctrl+t    send a shortcut"
	@echo "make wbox-type T=hello     type text (ASCII)"
	@echo "make wbox-scroll X= Y= N=-3  mouse wheel, negative = up"
	@echo "make wbox-log             last lines of the compositor/app log"
	@echo ""
	@echo "something to attach to, no tokens (tmux sessions tvty-fake / tvty-sim):"
	@echo "make fake-up              fake-claude in tmux session tvty-fake"
	@echo "make sim-up               simai-cli replaying dev/sim/dense.txt in tvty-sim"
	@echo "make flood-up             tvty-flood: 5 s of lines at full speed on Enter"
	@echo "make fake-down            kill them all"
	@echo ""
	@echo "a throwaway aiball with a demo project, for the ticket panel:"
	@echo "make emoji-font       colour emoji in the terminals (downloads Noto Color Emoji)"
	@echo "make install-desktop  Terminal Velocity in the desktop's launcher, with its icon (BIN= the binary)"
	@echo "make aiball-up / aiball-down   (point tvty at it with AIBALL_SOCK, see docs/TESTING.md)"

public-check:
	scripts/check-public

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

wbox-scroll:
	$(WBOX_CTL) scroll $(WBOX_CONFIG) $(X) $(Y) $(N)

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

# ── a throwaway aiball ───────────────────────────────────────────────────────
aiball-up:
	scripts/fake-aiball up

aiball-down:
	scripts/fake-aiball down

# Colour emoji in the terminals: Noto Color Emoji in bitmaps (CBDT), the kind
# GPUI colours, from Google's noto-emoji repository (OFL).
EMOJI_FONT_URL = https://raw.githubusercontent.com/googlefonts/noto-emoji/v2.047/fonts/NotoColorEmoji.ttf
EMOJI_FONT_DIR = $(or $(XDG_DATA_HOME),$(HOME)/.local/share)/tvty/fonts
emoji-font:
	mkdir -p "$(EMOJI_FONT_DIR)"
	curl -fsSL -o "$(EMOJI_FONT_DIR)/NotoColorEmoji.ttf" "$(EMOJI_FONT_URL)"
	@echo "colour emoji font in $(EMOJI_FONT_DIR): restart tvty"

# The launcher and the icon, for this user (no sudo). The launcher runs BIN:
# the debug build by default.
DATA_DIR = $(or $(XDG_DATA_HOME),$(HOME)/.local/share)
BIN ?= $(CURDIR)/target/debug/tvty
ICON_SIZES = 16 24 32 48 64 128 256 512

install-desktop:
	install -Dm644 assets/tvty.svg "$(DATA_DIR)/icons/hicolor/scalable/apps/tvty.svg"
	@if command -v magick >/dev/null; then \
		for s in $(ICON_SIZES); do \
			mkdir -p "$(DATA_DIR)/icons/hicolor/$${s}x$${s}/apps"; \
			magick -background none -density 384 assets/tvty.svg -resize $${s}x$${s} "$(DATA_DIR)/icons/hicolor/$${s}x$${s}/apps/tvty.png"; \
		done; \
	else echo "no ImageMagick: the scalable icon only"; fi
	mkdir -p "$(DATA_DIR)/applications"
	sed 's|@EXEC@|$(BIN)|' packaging/linux/tvty.desktop >"$(DATA_DIR)/applications/tvty.desktop"
	-gtk-update-icon-cache -q -t "$(DATA_DIR)/icons/hicolor" 2>/dev/null
	-update-desktop-database -q "$(DATA_DIR)/applications" 2>/dev/null
	@echo "Terminal Velocity installed: it runs $(BIN)"

uninstall-desktop:
	rm -f "$(DATA_DIR)/applications/tvty.desktop" "$(DATA_DIR)/icons/hicolor/scalable/apps/tvty.svg"
	for s in $(ICON_SIZES); do rm -f "$(DATA_DIR)/icons/hicolor/$${s}x$${s}/apps/tvty.png"; done
