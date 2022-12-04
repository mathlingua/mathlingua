
.PHONY: build
build:
	go build -o testbed/mlg main.go

.PHONY: all
all:
	make web && make

.PHONY: run
run:
	go run main.go

.PHONY: test
test:
	go test ./...

.PHONY: setupgo
setup:
	curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b ~/go/bin v1.49.0

.PHONY: vet
vet:
	go vet ./...

.PHONY: lint
lint:
	golangci-lint run

.PHONY: clean
clean:
	go clean

.PHONY: format
format:
	go fmt ./...

.PHONY: deps
deps:
	go mod download

.PHONY: testbed
testbed:
	go build -o testbed/mlg main.go && cd testbed && ./mlg $(args)

.PHONY: debug
debug:
	go run main.go debug $(args)

.PHONY: setupweb
setupweb:
	cd web && yarn install

.PHONY: web
web:
	cd web && yarn build
