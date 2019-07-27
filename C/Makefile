
CC=gcc
CFLAGS=-Wall -Wextra -Werror -I$(OUT_DIR)/lightning-prefix/include

LIBDIR=$(PREFIX)/lib
OBJDIR=$(OUT_DIR)/sys-bin

OBJS=$(OBJDIR)/register.o
TARGET=$(LIBDIR)/liblightningsys.a


$(TARGET): $(OBJS)
	ar cr $@ $?

$(OBJDIR)/%.o: %.c | $(OBJDIR)
	$(CC) $(CFLAGS) $^ -c -o $@

$(OBJDIR):
	mkdir -p $@



