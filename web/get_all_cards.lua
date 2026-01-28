wrk.method = "GET"
wrk.path = "/card/get_all_cards"

wrk.headers["Authorization"] =
  "v4.local.yQyyt0sDlWYORGWN8_bmt2qQWpzp6MMyJjQU85IwyChnWOQuy8-JxDJBoAADVuGOTNfELZBHpUmFTKFQPIlkvExTlQ7XgfCscVmV1gv7VBCGp2WPpbxHIggFzyMotMyRWpoYurpV9yqV1QKEbjCFxzsKn0CeKIgUSFYl7tPCVzxxgJclOOyN7Qmn3KCetLxaf-aCto0kBPLRZY9q0Scd-ybZpTcZdZDd97Pn_PMaiNveHdtBkup61mzyVB-b-KpFvdSlXyJfdosjVu9YZqgYeo-hx6HUuQl9ug"

wrk.headers["Content-Type"] = "text/plain"

wrk.body = [[
{
  "card_id": "-xJfpVhEfSi76XCO7SLaR",
  "amount_due": 13590
}
]]

