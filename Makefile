
.PHONY: build
build:
	go build ./...

.PHONY: run
run:
	go run main.go

.PHONY: test
test:
	go test ./...

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
