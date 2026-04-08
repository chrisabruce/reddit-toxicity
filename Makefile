IMAGE     := reddit-toxicity
CONTAINER := reddit-toxicity
PORT      ?= 3000
HOST      ?= 0.0.0.0
BIN        = target/release/reddit-toxicity
BIN_COMPACT = target/compact/reddit-toxicity

.PHONY: build build-compact run stop restart logs status clean size mem

## Build release binary
build-bin:
	cargo build -p reddit-toxicity-server --release

## Build smallest possible binary (LTO + strip + opt-level=z)
build-compact:
	cargo build -p reddit-toxicity-server --profile compact
	@echo ""
	@ls -lh $(BIN_COMPACT)
	@echo ""
	@echo "Binary: $(BIN_COMPACT)"

## Show binary sizes for both profiles
size: build-bin build-compact
	@echo ""
	@echo "=== Binary sizes ==="
	@printf "  release: " && ls -lh $(BIN) | awk '{print $$5}'
	@printf "  compact: " && ls -lh $(BIN_COMPACT) | awk '{print $$5}'

## Show memory usage of the running server
mem:
	@PID=$$(pgrep -f reddit-toxicity); \
	if [ -z "$$PID" ]; then \
		echo "Server is not running."; \
		exit 1; \
	fi; \
	echo "PID: $$PID"; \
	ps -o pid,rss,vsz,command -p $$PID | head -2; \
	echo ""; \
	RSS=$$(ps -o rss= -p $$PID | tr -d ' '); \
	echo "RSS (physical memory): $$(echo "scale=1; $$RSS / 1024" | bc) MB"

## Build the Docker image
build:
	docker build -t $(IMAGE) .

## Start the container (detached)
run: build
	@if docker ps -a --format '{{.Names}}' | grep -q '^$(CONTAINER)$$'; then \
		echo "Container already exists. Run 'make stop' first or 'make restart'."; \
		exit 1; \
	fi
	docker run -d --name $(CONTAINER) -p $(PORT):3000 \
		-e HOST=$(HOST) -e PORT=3000 \
		-e REDDIT_CLIENT_ID=$(REDDIT_CLIENT_ID) \
		-e REDDIT_CLIENT_SECRET=$(REDDIT_CLIENT_SECRET) \
		$(IMAGE)
	@echo "Started on http://localhost:$(PORT)"

## Stop and remove the container
stop:
	-docker stop $(CONTAINER) 2>/dev/null
	-docker rm $(CONTAINER) 2>/dev/null
	@echo "Stopped."

## Restart (stop then run)
restart: stop run

## Tail container logs
logs:
	docker logs -f $(CONTAINER)

## Show container status
status:
	@docker ps -a --filter name=^$(CONTAINER)$$ --format "table {{.Status}}\t{{.Ports}}"

## Remove the Docker image
clean: stop
	-docker rmi $(IMAGE) 2>/dev/null
	@echo "Cleaned."
