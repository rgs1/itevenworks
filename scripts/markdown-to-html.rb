#!/usr/bin/env ruby

require 'redcarpet'
require 'rouge'

class HTML < Redcarpet::Render::HTML
  def block_code(code, language)
    highlight(code, language || 'text')
  end

  private
  def highlight(text, lexer)
    lexer = Rouge::Lexer.find(lexer) unless lexer.respond_to? :lex
    raise "unknown lexer #{lexer}" unless lexer

    if lexer.tag == 'make'
      text.gsub! /^    /, "\t"
    end

    opts = {
      css_class: "highlight #{lexer.tag}",
      wrap: true,
      line_numbers: true,
      inline_theme: "github"
    }
    formatter = Rouge::Formatters::HTML.new(opts)

    formatter.format(lexer.lex(text))
  end
end

def markdown(text)
  render_options = {
    filter_html:     false,
    hard_wrap:       true,
    link_attributes: { rel: 'nofollow' },
  }
  renderer = HTML.new(render_options)

  extensions = {
    autolink:           true,
    fenced_code_blocks: true,
    lax_spacing:        true,
    no_intra_emphasis:  true,
    strikethrough:      true,
    superscript:        true,
  }
  Redcarpet::Markdown.new(renderer, extensions).render(text)
end

header = <<HEADER
<!DOCTYPE html>
<html>
<head>
<title>itevenworks.net!</title>
<link data-turbolinks-track="true" href="/assets/rouge.css" media="all" rel="stylesheet" />
</head>
<body>
HEADER
body = STDIN.readlines().join('')
footer = '</body></html>'
puts header + markdown(body) + footer
