# name of your application
APPLICATION = mailr-example

# If no BOARD is found in the environment, use this default:
BOARD ?= native

# This has to be the absolute path to the RIOT base directory:
RIOTBASE ?= $(CURDIR)/../../../../RIOT

# Comment this out to disable code in RIOT that does safety checking
# which is not needed in a production environment but helps in the
# development process:
DEVELHELP ?= 1

# Change this to 0 show compiler invocation lines by default:
QUIET ?= 1

# Change these to configure the SMTP example

# Using global addr may help if cannot connect through local-scope ipv6
# SMTP_SERVER_IPV6_ADDR ?= dead:beef::1
# ADD_LOCAL_IPV6_ADDR ?= dead:beef::2

SMTP_SERVER_IPV6_ADDR ?= fe80::5844:b4ff:fe1c:b066
SMTP_SERVER_PORT ?= 2525
BUFFER_SIZE ?= 1024

CFLAGS += -DSMTP_SERVER_IPV6_ADDR=\"$(SMTP_SERVER_IPV6_ADDR)\"
CFLAGS += -DSMTP_SERVER_PORT=$(SMTP_SERVER_PORT)
CFLAGS += -DBUFFER_SIZE=$(BUFFER_SIZE)

ifneq (,$(ADD_LOCAL_IPV6_ADDR))
  CFLAGS += -DADD_LOCAL_IPV6_ADDR=\"$(ADD_LOCAL_IPV6_ADDR)\"
endif

# Modules to include:
USEMODULE += netdev_default
USEMODULE += auto_init_gnrc_netif
USEMODULE += gnrc_ipv6_default
USEMODULE += gnrc_icmpv6_echo

EXTERNAL_MODULE_DIRS += $(CURDIR)/..

USEMODULE += mailr_riot_module

include $(RIOTBASE)/Makefile.include

# If building in Docker, use the external module dir path relative to the project (git repo) instead
# of the default `$(DOCKER_BUILD_ROOT)/external`. This is required for the `mailr-nal` Cargo dependency
# relative path (`../../../`) in `$(DOCKER_RIOTPROJECT)/examples/RIOT/mailr_riot_module` to work.
DOCKER_OVERRIDE_CMDLINE += 'EXTERNAL_MODULE_DIRS=$(DOCKER_RIOTPROJECT)/examples/RIOT'
