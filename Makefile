
.PHONY: build
build:
	make union && go build -o bin/mlg main.go

.PHONY: union
union:
	go run unions/main.go

.PHONY: web
web:
	cd web && yarn build

.PHONY: all
all:
	make web && make

.PHONY: install
install:
	make build && cp bin/mlg /usr/local/bin/

.PHONY: run
run:
	go run main.go

.PHONY: test
test:
	go test ./...

.PHONY: testv
testv:
	go test ./... -v

.PHONY: release
release:
	./scripts/build-releases.sh --notarize

.PHONY: artifacts
artifacts:
	./scripts/build-artifacts.sh

.PHONY: format
format:
	go fmt ./...

.PHONY: vet
vet:
	go vet ./...

.PHONY: lint
lint:
	golangci-lint run --skip-files='\.\./.*'

.PHONY: longlines
longlines:
	./scripts/find-long-lines.sh

.PHONY: check
check:
	make format && make vet && make lint && make longlines

.PHONY: clean
clean:
	go clean && rm -Rf web/build

.PHONY: deps
deps:
	go mod download

.PHONY: testbed
testbed:
	go build -o testbed/mlg main.go && cd testbed && ./mlg $(args)

.PHONY: debug
debug:
	go run main.go debug $(args)

.PHONY: setup
setup:
	make setupgo && make setupweb

.PHONY: setupgo
setupgo:
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest

.PHONY: setupweb
setupweb:
	cd web && yarn install
