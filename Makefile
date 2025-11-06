# Makefile for build & compose operations
# Usage:
#   make build           # build images (optionally SERVICE=<name>)
#   make compose         # docker-compose up -d (builds if needed)
#   make down            # docker-compose down
#   make restart         # down then up
#   make logs            # follow logs (optionally SERVICE=<name>)
#   make clean           # remove containers, images and volumes
#
# Variables:
#   COMPOSE_FILE (default: docker-compose.yml)
#   SERVICE (optional, restrict commands to a single service)

COMPOSE_FILE ?= docker-compose.yml
SERVICE ?=
DC ?= docker-compose

.PHONY: all build compose up down restart logs ps clean

all: compose

build:
	$(DC) -f $(COMPOSE_FILE) build $(SERVICE)

compose:
	$(DC) -f $(COMPOSE_FILE) up -d --remove-orphans $(SERVICE)

up: compose

down:
	$(DC) -f $(COMPOSE_FILE) down

restart: down up

logs:
	$(DC) -f $(COMPOSE_FILE) logs -f $(SERVICE)

ps:
	$(DC) -f $(COMPOSE_FILE) ps

clean:
	$(DC) -f $(COMPOSE_FILE) down --rmi all --volumes --remove-orphans