vim.lsp.config(
    "ruff",
    {
        on_attach = function(client, bufnr)
            if client.supports_method("textDocument/formatting") then
                vim.api.nvim_create_autocmd(
                    "BufWritePre",
                    {
                        group = vim.api.nvim_create_augroup("LspFormattingRuff", {clear = true}),
                        buffer = bufnr,
                        callback = function()
                            vim.lsp.buf.format({bufnr = bufnr})
                            vim.lsp.buf.code_action(
                                {
                                    bufnr = bufnr,
                                    context = {
                                        only = {"source.organizeImports"}
                                    },
                                    apply = true
                                }
                            )
                        end
                    }
                )
            end
        end
    }
)

vim.lsp.config(
    "rust_analyzer",
    {
        settings = {
            ["rust-analyzer"] = {
                diagnostics = {
                    enable = false;
                }
            }
        },
        on_attach = function (client, bufnr)
            if client.server_capabilities.documentFormattingProvider then
                vim.api.nvim_create_autocmd(
                    "BufWritePre",
                    {
                        group = vim.api.nvim_create_augroup("LspFormatting", { clear = true }),
                        buffer = bufnr,
                        callback = function()
                            vim.lsp.buf.format({ bufnr = bufnr })
                        end,
                    }
                )
            end
        end,
    }
)

vim.lsp.enable("ruff")

vim.lsp.enable("rust_analyzer")

vim.lsp.enable("ty")
